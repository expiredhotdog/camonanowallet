use super::error::CliError;
use super::types::Amount;
use super::CliClient;
use client::{Account, CamoAccount, Client, Receivable};

fn get_display_balance(client: &Client, account: &Account) -> String {
    let amount: Amount = client
        .frontiers_db
        .account_balance(account)
        .unwrap_or(0)
        .into();
    amount.to_string()
}

/// Returns `Vec<(index, account)>`, sorted
fn get_normal_accounts(client: &Client) -> Vec<(u32, Account)> {
    let mut accounts: Vec<(u32, Account)> = client
        .wallet_db
        .account_db
        .all_infos()
        .iter()
        .map(|info| (info.index, info.account.clone()))
        .collect();
    accounts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    accounts
}

/// Returns `Vec<(index, account)>`, sorted
fn get_camo_accounts(client: &Client) -> Vec<(u32, CamoAccount)> {
    let mut accounts: Vec<(u32, CamoAccount)> = client
        .wallet_db
        .camo_account_db
        .all_infos()
        .iter()
        .map(|info| (info.index, info.account.clone()))
        .collect();
    accounts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    accounts
}

fn get_derived_accounts(client: &Client, account: &CamoAccount) -> Vec<Account> {
    client
        .wallet_db
        .derived_account_db
        .get_info_from_master(&client.wallet_db.camo_account_db, account)
        .iter()
        .map(|info| info.account.clone())
        .collect()
}

fn filter_receivable(receivables: &[&Receivable], account: &Account) -> Amount {
    receivables
        .iter()
        .filter(|receivable| &receivable.recipient == account)
        .map(|receivable| receivable.amount)
        .sum::<u128>()
        .into()
}

fn print_balance(receivable: Amount, s: String) {
    match receivable.value > 0 {
        true => println!("{s} (+ {receivable} Nano receivable)",),
        false => println!("{s}"),
    }
}

pub fn execute(cli_client: &CliClient) -> Result<(), CliError> {
    let client = &cli_client.internal;
    let receivables: Vec<&Receivable> = cli_client.cached_receivable.values().collect();

    // total balance
    let total: Amount = client.wallet_balance().into();
    let total_receivable: Amount = receivables
        .iter()
        .map(|receivable| receivable.amount)
        .sum::<u128>()
        .into();
    print_balance(total_receivable, format!("total: {total} Nano"));

    // normal accounts
    for (index, account) in get_normal_accounts(client) {
        let balance = get_display_balance(client, &account);
        let account_receivable = filter_receivable(&receivables, &account);
        print_balance(
            account_receivable,
            format!("{account} (#{index}): {balance} Nano"),
        );
    }

    // camo accounts
    for (index, camo_account) in get_camo_accounts(client) {
        println!("{camo_account} (#{index}):");

        // main account
        let main_account = camo_account.signer_account();
        let balance = get_display_balance(client, &main_account);
        let account_receivable = filter_receivable(&receivables, &main_account);
        print_balance(
            account_receivable,
            format!("\t{main_account} (main): {balance} Nano"),
        );

        // derived accounts
        for account in get_derived_accounts(client, &camo_account) {
            let balance = get_display_balance(client, &account);
            let account_receivable = filter_receivable(&receivables, &account);
            print_balance(account_receivable, format!("\t{account}: {balance} Nano"));
        }
    }
    Ok(())
}
