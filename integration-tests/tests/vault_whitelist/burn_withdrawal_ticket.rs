#[cfg(test)]
mod tests {
    use solana_sdk::{instruction::InstructionError, signature::Keypair, signer::Signer};

    use crate::{
        client::vault_client::VaultStakerWithdrawalTicketRoot,
        fixtures::{
            assert_ix_error,
            fixture::{ConfiguredVault, TestBuilder},
        },
    };

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_ok() {
        const MINT_AMOUNT: u64 = 100_000;
        const DEPOSIT_FEE_BPS: u16 = 100;
        const WITHDRAWAL_FEE_BPS: u16 = 100;

        let deposit_fee_bps = DEPOSIT_FEE_BPS;
        let withdrawal_fee_bps = WITHDRAWAL_FEE_BPS;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            mut vault_whitelist_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        vault_whitelist_client.do_initialize_config().await.unwrap();

        vault_whitelist_client
            .do_initialize_whitelist(&vault_root)
            .await
            .unwrap();

        vault_whitelist_client
            .do_set_mint_burn_admin(&vault_root)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        vault_whitelist_client
            .do_add_to_whitelist(&vault_root, &depositor.pubkey())
            .await
            .unwrap();

        let min_amount_out: u64 = 90000;

        vault_whitelist_client
            .do_mint(&vault_root, &vault, &depositor, MINT_AMOUNT, min_amount_out)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        let operator_root_pubkeys: Vec<_> = operator_roots
            .iter()
            .map(|root| root.operator_pubkey)
            .collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let operator_root = operator_roots.first().unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_root.operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault_operator_delegation.delegation_state.staked_amount(),
            MINT_AMOUNT
        );

        // the user is withdrawing 99,000 VRT tokens, there is a 1% fee on withdraws, so
        // 98010 tokens will be undeleged for withdraw
        let amount_to_dequeue = MINT_AMOUNT * (10_000 - WITHDRAWAL_FEE_BPS) as u64 / 10_000;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_whitelist_client
            .do_enqueue_withdrawal(&vault_root, &vault, &depositor, amount_to_dequeue)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        vault_whitelist_client
            .do_burn_withdrawal_ticket(&config, &vault_root, &vault, &depositor, &base)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_invalid_user() {
        const MINT_AMOUNT: u64 = 100_000;
        const DEPOSIT_FEE_BPS: u16 = 100;
        const WITHDRAWAL_FEE_BPS: u16 = 100;

        let deposit_fee_bps = DEPOSIT_FEE_BPS;
        let withdrawal_fee_bps = WITHDRAWAL_FEE_BPS;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            mut vault_whitelist_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        vault_whitelist_client.do_initialize_config().await.unwrap();

        vault_whitelist_client
            .do_initialize_whitelist(&vault_root)
            .await
            .unwrap();

        vault_whitelist_client
            .do_set_mint_burn_admin(&vault_root)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        vault_whitelist_client
            .do_add_to_whitelist(&vault_root, &depositor.pubkey())
            .await
            .unwrap();

        let min_amount_out: u64 = 90000;

        vault_whitelist_client
            .do_mint(&vault_root, &vault, &depositor, MINT_AMOUNT, min_amount_out)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        let operator_root_pubkeys: Vec<_> = operator_roots
            .iter()
            .map(|root| root.operator_pubkey)
            .collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let operator_root = operator_roots.first().unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_root.operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault_operator_delegation.delegation_state.staked_amount(),
            MINT_AMOUNT
        );

        // the user is withdrawing 99,000 VRT tokens, there is a 1% fee on withdraws, so
        // 98010 tokens will be undeleged for withdraw
        let amount_to_dequeue = MINT_AMOUNT * (10_000 - WITHDRAWAL_FEE_BPS) as u64 / 10_000;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base: _ } = vault_whitelist_client
            .do_enqueue_withdrawal(&vault_root, &vault, &depositor, amount_to_dequeue)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let invalid_depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &invalid_depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        let result = vault_whitelist_client
            .do_enqueue_withdrawal(&vault_root, &vault, &invalid_depositor, amount_to_dequeue)
            .await;

        assert_ix_error(result, InstructionError::InvalidAccountOwner);
    }
}
