use solana_rpcx_sdk::prelude::*;
use once_cell::sync::Lazy;
use tentacles::state::{SplitWallet};
use tentacles::ID as TENTACLES_PROGRAM_ID;

fn split_wallet_to_json(wallet: &SplitWallet) -> Result<String, String> {
    let members_json: Vec<_> = wallet.members.iter().map(|m| {
        serde_json::json!({
            "member": m.member.to_string(),
            "shares": m.shares,
            "disburse_cycles": m.disburse_cycles,
            "member_token_account": m.member_token_account.to_string(),
        })
    }).collect();
    
    let json = serde_json::json!({
        "authority": wallet.authority.to_string(),
        "tentacles": wallet.tentacles.to_string(),
        "name": wallet.name,
        "mint": wallet.mint.to_string(),
        "total_shares": wallet.total_shares,
        "total_members": wallet.total_members,
        "last_inflow": wallet.last_inflow,
        "total_inflow": wallet.total_inflow,
        "token_account": wallet.token_account.to_string(),
        "remaining_flow": wallet.remaining_flow,
        "bump_seed": wallet.bump_seed,
        "total_available_shares": wallet.total_available_shares,
        "disburse_cycles": wallet.disburse_cycles,
        "members": members_json,
    });
    
    serde_json::to_string(&json).map_err(|e| e.to_string())
}

static PARSER: Lazy<Parser> = Lazy::new(|| {
    ParserBuilder::new(TENTACLES_PROGRAM_ID.to_string())
        .register_anchor_account::<SplitWallet, _>(split_wallet_to_json)
        .with_metadata(ProgramMetadata {
            name: Some("Tentacles".to_string()),
            program_id: Some(TENTACLES_PROGRAM_ID.to_string()),
            project_url: Some("https://github.com/Ozodimgba/tentacles".to_string()),
            version: Some("1.0.0".to_string()),
        })
        .build()
});

struct Component;

impl ProgramParserGuest for Component {
    fn parse_account(account: SolanaAccount) -> Result<ParsedAccount, ParseError> {
        PARSER.parse_account(&account)
    }
    
    fn parse_accounts(accounts: Vec<SolanaAccount>) 
        -> Result<Vec<Result<ParsedAccount, ParseError>>, String> 
    {
        Ok(accounts.into_iter().map(|a| PARSER.parse_account(&a)).collect())
    }
    
    fn parse_with_options(account: SolanaAccount, options: ParseOptions) 
        -> Result<ParsedAccount, ParseError> 
    {
        let mut result = PARSER.parse_account(&account)?;
        
        if options.pretty_json {
            let value: serde_json::Value = serde_json::from_str(&result.data)
                .map_err(|e| ParseError::InvalidData(e.to_string()))?;
            result.data = serde_json::to_string_pretty(&value)
                .map_err(|e| ParseError::InvalidData(e.to_string()))?;
        }
        
        Ok(result)
    }
    
    fn parse_instruction(instruction: InstructionData) 
        -> Result<ParsedInstruction, ParseError> 
    {
        PARSER.parse_instruction(&instruction)
    }
    
    fn parse_instructions(instructions: Vec<InstructionData>) 
        -> Result<Vec<Result<ParsedInstruction, ParseError>>, String> 
    {
        Ok(instructions.into_iter().map(|i| PARSER.parse_instruction(&i)).collect())
    }
    
    fn can_parse(owner: String, data_preview: Vec<u8>) -> bool {
        PARSER.can_parse(&owner, &data_preview)
    }
    
    fn get_supported_types() -> Vec<String> {
        PARSER.get_supported_types()
    }
    
    fn get_program_metadata() -> Option<ProgramMetadata> {
        PARSER.get_metadata()
    }
}

impl AccountsTransformerGuest for Component {
    fn transform_accounts(_accounts: Vec<SolanaAccount>, _params: String) -> Result<String, String> {
        Err("Not implemented".to_string())
    }
}

impl AccountsTransformerSetupGuest for Component {
    fn setup() -> TransformerRequest {
        TransformerRequest { 
            seeds: vec![], 
            addresses: None, 
            owner_filter: None 
        }
    }
}

impl TransactionTransformerGuest for Component {
    fn transform_transaction(_tx: SolanaTransaction, _params: String) -> Result<String, String> {
        Err("Not implemented".to_string())
    }
}

impl ViewFunctionGuest for Component {
    fn view(_method: String, _params: String) -> Result<String, String> {
        Err("Not implemented".to_string())
    }
}

solana_rpcx_sdk::bindings::export!(Component with_types_in solana_rpcx_sdk::bindings);


// static PARSER: Lazy<Parser> = Lazy::new(|| {
//     ParserBuilder::new(TENTACLES_PROGRAM_ID.to_string())
//         .register_custom_account(
//             "SplitWallet",
//             Some(compute_anchor_discriminator("account", "SplitWallet").to_vec()),
//             |data: &[u8]| {
//                 if data.len() < 8 {
//                     return Err(ParseError::InsufficientData("Too short".to_string()));
//                 }
                
//                 let discriminator = compute_anchor_discriminator("account", "SplitWallet");
//                 if &data[0..8] != &discriminator {
//                     return Err(ParseError::UnknownAccountType("Wrong discriminator".to_string()));
//                 }
                
//                 // Use AnchorDeserialize, need mutable reference to slice
//                 let mut data_slice = &data[8..];
//                 let wallet = SplitWallet::try_deserialize(&mut data_slice)
//                     .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
                
//                 let json = split_wallet_to_json(&wallet)
//                     .map_err(|e| ParseError::InvalidData(e))?;
                
//                 Ok(ParsedAccount {
//                     account_type: "SplitWallet".to_string(),
//                     data: json,
//                     discriminator: Some(discriminator.to_vec()),
//                 })
//             }
//         )
//         .with_metadata(ProgramMetadata {
//             name: Some("Tentacles".to_string()),
//             program_id: Some(TENTACLES_PROGRAM_ID.to_string()),
//             project_url: Some("https://github.com/Ozodimgba/tentacles".to_string()),
//             version: Some("1.0.0".to_string()),
//         })
//         .build()
// });