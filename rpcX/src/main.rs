// Complete example: Fetch from RPC, decode base64, parse with WASM component

use anyhow::{Result, anyhow};
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

wasmtime::component::bindgen!({
    path: "../solana-rpcx-bindings/wit",
    world: "full-parser",
});

struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl component::solana_rpcx_bindings::accounts_db::Host for HostState {
    fn get_account(
        &mut self,
        pubkey: String,
    ) -> Option<component::solana_rpcx_bindings::types::SolanaAccount> {
        println!("Host: get_account called for {}", pubkey);
        None
    }
    
    fn get_multiple_accounts(
        &mut self,
        pubkeys: Vec<String>,
    ) -> Vec<Option<component::solana_rpcx_bindings::types::SolanaAccount>> {
        vec![None; pubkeys.len()]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
  
    println!("Setting up WASM runtime...");
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    component::solana_rpcx_bindings::accounts_db::add_to_linker(
        &mut linker, 
        |state: &mut HostState| state
    )?;
    
    let component = Component::from_file(
        &engine, 
        "../target/wasm32-wasip1/release/tentacles_parser.wasm"
    )?;
    
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let state = HostState { 
        wasi,
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    
    let instance = FullParser::instantiate(&mut store, &component, &linker)?;
    let parser = instance.component_solana_rpcx_bindings_program_parser();
    println!("WASM component ready\n");
    
    println!("Fetching account from Solana RPC...");
    
    // You can replace this with any Tentacles SplitWallet address
    let account_pubkey = std::env::var("ACCOUNT_ADDRESS")
        .unwrap_or_else(|_| "FgH8NKRZ16MRQgogZrdKSMqMhr4gnZSVgHBSU92hMnzh".to_string());
    
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string());
    
    println!("  RPC: https://mainnet.helius-rpc.com");
    println!("  Account: {}\n", account_pubkey);
    
    // Fetch account info
    let rpc_account = fetch_account_from_rpc(&rpc_url, &account_pubkey).await?;
    println!("Account fetched: {} bytes, {} lamports", 
        rpc_account.data.len(), rpc_account.lamports);
    
    let wasm_account = component::solana_rpcx_bindings::types::SolanaAccount {
        pubkey: account_pubkey.clone(),
        data: rpc_account.data,
        owner: rpc_account.owner,
        lamports: rpc_account.lamports,
        executable: rpc_account.executable,
        rent_epoch: rpc_account.rent_epoch,
    };

    println!("Parsing account with WASM component...\n");
    
    match parser.call_parse_account(&mut store, &wasm_account)? {
        Ok(parsed) => {
            println!("Successfully parsed!\n");
            println!("Account Type: {}\n", parsed.account_type);
            
            // Pretty print the parsed data
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&parsed.data) {
                println!("Parsed Account Data:");
                println!("{}\n", serde_json::to_string_pretty(&json)?);
            }
        }
        Err(e) => {
            println!("Parse error: {:?}\n", e);
        }
    }
    
    println!("✅ Complete!");
    
    Ok(())
}


// helpers
#[derive(Debug)]
struct RpcAccount {
    pub lamports: u64,
    pub owner: String,
    pub data: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: u64,
}

/// Fetch account from Solana RPC and decode base64 data
async fn fetch_account_from_rpc(rpc_url: &str, pubkey: &str) -> Result<RpcAccount> {
    let client = reqwest::Client::new();
    
    // Build RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            pubkey,
            {
                "encoding": "base64",
                "commitment": "confirmed"
            }
        ]
    });
    
    // Send request
    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    // Parse response
    let value = json
        .get("result")
        .and_then(|r| r.get("value"))
        .ok_or_else(|| anyhow!("Account not found or RPC error"))?;
    
    // Check if account exists
    if value.is_null() {
        return Err(anyhow!("Account does not exist"));
    }
    
    // Extract fields
    let lamports = value["lamports"].as_u64()
        .ok_or_else(|| anyhow!("Missing lamports"))?;
    
    let owner = value["owner"].as_str()
        .ok_or_else(|| anyhow!("Missing owner"))?
        .to_string();
    
    let executable = value["executable"].as_bool().unwrap_or(false);
    let rent_epoch = value["rentEpoch"].as_u64().unwrap_or(0);
    
    // Decode base64 data
    let data_array = value["data"].as_array()
        .ok_or_else(|| anyhow!("Missing data array"))?;
    
    let base64_data = data_array[0].as_str()
        .ok_or_else(|| anyhow!("Missing base64 data"))?;
    
    let data = general_purpose::STANDARD.decode(base64_data)?;
    
    Ok(RpcAccount {
        lamports,
        owner,
        data,
        executable,
        rent_epoch,
    })
}

#[allow(dead_code)]
fn parse_from_json(json_str: &str) -> Result<RpcAccount> {
    let json: serde_json::Value = serde_json::from_str(json_str)?;
    
    let value = json["result"]["value"]
        .as_object()
        .ok_or_else(|| anyhow!("Invalid JSON structure"))?;
    
    let lamports = value["lamports"].as_u64()
        .ok_or_else(|| anyhow!("Missing lamports"))?;
    
    let owner = value["owner"].as_str()
        .ok_or_else(|| anyhow!("Missing owner"))?
        .to_string();
    
    let executable = value["executable"].as_bool().unwrap_or(false);
    let rent_epoch = value["rentEpoch"].as_u64().unwrap_or(0);
    
    let data_array = value["data"].as_array()
        .ok_or_else(|| anyhow!("Missing data array"))?;
    
    let base64_data = data_array[0].as_str()
        .ok_or_else(|| anyhow!("Missing base64 data"))?;
    
    let data = general_purpose::STANDARD.decode(base64_data)?;
    
    Ok(RpcAccount {
        lamports,
        owner,
        data,
        executable,
        rent_epoch,
    })
}