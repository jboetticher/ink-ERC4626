#![cfg_attr(not(feature = "std"), no_std, no_main)]

// TODO: use Zeitgeist balances instead
use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

/*

An implementation of ERC4626, a standardization of minting, burning, and redeeming a token
for other assets.

Since this is an ink! smart contract, the complete implementation isn't available for your
specific parachain, as you may require specific pallets to work with specific assets.
Please look for @dev tags to see where manual implementation is necessary.

This smart contract was written and based off of the ERC20 smart contract provided by the
ink-examples repository.

*/

#[ink::contract]
mod erc4626_20 {
    use ink::storage::Mapping;
    use ink::env::Error as EnvError;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct Erc4626 {
        // vault_token: Erc20Ref,
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
        /// The decimals of the asset being represented
        decimals: u8,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        owner: AccountId,
        assets: Balance,
        shares: Balance,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        receiver: AccountId,
        #[ink(topic)]
        owner: AccountId,
        assets: Balance,
        shares: Balance,
    }

    /// The ERC-20 ErcError types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ErcError {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Returned when making a deposit, and the deposit is too high.
        ExceededMaxDeposit,
        /// Returned when minting, and the mint is too high.
        ExceededMaxMint,
        /// Returned when withdrawing, and the withdrawl is too high.
        ExceededMaxWithdraw,
        /// Returned when redeeming, and the redeem is too high.
        ExceededMaxRedeem,
        CallRuntimeFailed
    }

    impl From<EnvError> for ErcError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => ErcError::CallRuntimeFailed,
                _ => panic!("Unexpected ErcError from `pallet-contracts`."),
            }
        }
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, ErcError>;

    impl Erc4626 {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
                decimals: 10,         // Decimals is 10 because ZTG is 10
                allowances: Default::default(),
                // vault_token: vaulted
            }
        }

        // region: Read Only

        // The address/multilocation of the underlying token
        // used for the vault for accounting, depositing, withdrawing.
        #[ink(message)]
        pub fn asset(&self) -> crate::ZeitgeistAsset {
            crate::ZeitgeistAsset::Ztg
        }

        #[ink(message)]
        pub fn total_assets(&self) -> Balance {
            self.env().balance()
        }

        /// Returns the amount of shares that would be exchanged by the vault for the
        /// amount of assets provided.
        #[ink(message)]
        pub fn convert_to_shares(&self, assets: Balance) -> Balance {
            assets * 10_u128.pow(self.decimal_offset().into())
        }

        /// returns the amount of assets that would be exchanged by the vault for the
        /// amount of shares provided.
        #[ink(message)]
        pub fn convert_to_assets(&self, shares: Balance) -> Balance {
            shares as u128 / 10_u128.pow(self.decimal_offset().into())
        }

        /// The maximum amount of underlying assets that can be deposited in a single
        /// deposit call by the receiver.
        #[ink(message)]
        pub fn max_deposit(&self, _depositor: AccountId) -> Balance {
            // @dev You can change this function to change the maximum amount that
            // can be deposited at a time
            Balance::from(u128::MAX)
        }

        /// Allows users to simulate the effects of their deposit at the current block.
        #[ink(message)]
        pub fn preview_deposit(&self, assets: Balance) -> Balance {
            // @dev You can change this function to change the calculation of depositing
            self.convert_to_shares(assets)
        }

        /// Returns the maximum amount of shares that can be minted in a single mint
        /// call by the receiver.
        #[ink(message)]
        pub fn max_mint(&self, _receiver: AccountId) -> Balance {
            // @dev You can change this function to change the maximum amount of shares
            // that can be minted at a time
            Balance::from(u128::MAX)
        }

        /// Returns the maximum amount of shares that can be minted in a single mint
        /// call by the receiver.
        #[ink(message)]
        pub fn preview_mint(&self, shares: Balance) -> Balance {
            // @dev You can change this function to change the calculation of minting
            self.convert_to_assets(shares)
        }

        /// Mints exactly shares vault shares to receiver by depositing assets of
        /// underlying tokens.
        #[ink(message)]
        pub fn max_withdraw(&self, _owner: AccountId) -> Balance {
            // @dev You can change this function to change the maximum amount of assets
            // that can be withdrawn at a time
            Balance::from(u128::MAX)
        }

        /// Allows users to simulate the effects of their withdrawal at the current block.
        #[ink(message)]
        pub fn preview_withdraw(&self, assets: Balance) -> Balance {
            // @dev You can change this function to change the calculation of withdrawing
            self.convert_to_shares(assets)
        }

        /// Returns the maximum amount of shares that can be redeemed from the owner balance
        /// through a redeem call.
        #[ink(message)]
        pub fn max_redeem(&self, _owner: AccountId) -> Balance {
            // @dev You can change this function to change the maximum amount of assets
            // that can be redeemed at a time
            Balance::from(u128::MAX)
        }

        /// Allows users to simulate the effects of their redemption at the current block.
        #[ink(message)]
        pub fn preview_redeem(&self, shares: Balance) -> Balance {
            // @dev You can change this function to change the calculation of redeeming
            self.convert_to_assets(shares)
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        /// Returns the decimals of this ERC20 asset.
        pub fn decimals(&self) -> u8 {
            self.decimals + self.decimal_offset()
        }

        /// Returns the decimal offset that this asset represents
        pub fn decimal_offset(&self) -> u8 {
            // @dev Change this to increase the ratio of tokens to ZTG
            1
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        // endregion

        // region: Inlines

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[inline]
        fn real_deposit(
            &mut self,
            _caller: AccountId,
            receiver: AccountId,
            assets: Balance,
            shares: Balance,
        ) -> Result<()> {
            // @dev Must implement the transfer of vaulted asset to this address (vault)

            // Mint
            let cur = self.balances.get(receiver).unwrap_or(0);
            self.balances.insert(receiver, &(cur + shares));
            self.total_supply += shares;

            self.env().emit_event(Deposit {
                sender: self.env().caller(),
                owner: receiver,
                assets,
                shares,
            });
            Ok(())
        }

        fn real_withdraw(
            &mut self,
            caller: AccountId,
            receiver: AccountId,
            owner: AccountId,
            assets: Balance,
            shares: Balance,
        ) -> Result<()> {
            // Spend allowance if necessary
            if caller != owner {
                let allowance = self.allowance_impl(&owner, &caller);
                if allowance < shares {
                    return Err(ErcError::InsufficientAllowance);
                }
                self.allowances
                    .insert((&owner, &caller), &(allowance - shares));
            }

            // Burn
            let cur = self.balances.get(owner).unwrap_or(0);
            if cur < shares {
                return Err(ErcError::InsufficientBalance);
            }
            self.balances.insert(owner, &(cur - shares));
            self.total_supply -= shares;

            // @dev Must implement the transfer of valuted asset to the receiver
            self.env()
                .call_runtime(&crate::RuntimeCall::AssetManager(
                    crate::AssetManagerCall::Transfer {
                        dest: receiver.into(),
                        currency_id: crate::ZeitgeistAsset::Ztg,
                        amount: assets,
                    },
                ))
                .map_err(Into::<ErcError>::into)?;

            self.env().emit_event(Withdraw {
                sender: caller,
                receiver,
                owner,
                assets,
                shares,
            });
            Ok(())
        }

        // endregion

        /// Deposits assets of underlying tokens into the vault and grants ownership of shares to receiver.
        #[ink(message, payable)]
        pub fn deposit(&mut self, assets: Balance, receiver: AccountId) -> Result<()> {
            if assets > self.max_deposit(self.env().caller()) {
                return Err(ErcError::ExceededMaxDeposit);
            }

            // Ensures that value is being transferred into the account
            if assets != self.env().transferred_value() {
                return Err(ErcError::InsufficientAllowance);
            }

            let shares = self.preview_deposit(assets);
            self.real_deposit(self.env().caller(), receiver, assets, shares)?;
            Ok(())
        }

        #[ink(message, payable)]
        pub fn mint(&mut self, shares: Balance, receiver: AccountId) -> Result<()> {
            if shares > self.max_mint(receiver) {
                return Err(ErcError::ExceededMaxMint);
            }

            let assets = self.preview_mint(shares);

            // Ensures that value is being transferred into the smart contract
            if assets != self.env().transferred_value() {
                return Err(ErcError::InsufficientAllowance);
            }

            self.real_deposit(self.env().caller(), receiver, assets, shares)?;
            Ok(())
        }

        /// Burns shares from owner and send exactly assets token from the vault to receiver.
        #[ink(message)]
        pub fn withdraw(
            &mut self,
            assets: Balance,
            receiver: AccountId,
            owner: AccountId,
        ) -> Result<()> {
            if assets > self.max_deposit(owner) {
                return Err(ErcError::ExceededMaxWithdraw);
            }

            let shares = self.preview_withdraw(assets);
            self.real_withdraw(self.env().caller(), receiver, owner, assets, shares)?;
            Ok(())
        }

        /// Burns shares from owner and send exactly assets token from the vault to receiver.
        #[ink(message)]
        pub fn redeem(
            &mut self,
            shares: Balance,
            receiver: AccountId,
            owner: AccountId,
        ) -> Result<()> {
            if shares > self.max_redeem(owner) {
                return Err(ErcError::ExceededMaxWithdraw);
            }

            let assets = self.preview_redeem(shares);
            self.real_withdraw(self.env().caller(), receiver, owner, assets, shares)?;
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` ErcError if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with
        /// `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` ErcError if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` ErcError if there are not enough tokens on
        /// the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(ErcError::InsufficientAllowance);
            }
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` ErcError if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(ErcError::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink::primitives::{Clear, Hash};

        type Event = <Erc4626 as ::ink::reflect::ContractEventBase>::Type;

        fn assert_transfer_event(
            event: &ink::env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Erc4626::Transfer",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc4626::Transfer::from",
                    value: &expected_from,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc4626::Transfer::to",
                    value: &expected_to,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc4626::Transfer::value",
                    value: &expected_value,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::CLEAR_HASH;
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {n}"
                );
            }
        }

        /// The default constructor does its job.
        #[ink::test]
        fn new_works() {
            // Constructor works.
            let _erc20 = Erc4626::new(100, 10);

            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
        }

        /// The total supply was applied.
        #[ink::test]
        fn total_supply_works() {
            // Constructor works.
            let erc20 = Erc4626::new(100, 10);
            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            // Get the token total supply.
            assert_eq!(erc20.total_supply(), 100);
        }

        /// The decimals was applied.
        #[ink::test]
        fn decimals_works() {
            // Constructor works.
            let erc20 = Erc4626::new(100, 10);
            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            // Get the decimals.
            assert_eq!(erc20.decimals(), 10);
        }

        /// Get the actual balance of an account.
        #[ink::test]
        fn balance_of_works() {
            // Constructor works
            let erc20 = Erc4626::new(100, 10);
            // Transfer event triggered during initial construction
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Alice owns all the tokens on contract instantiation
            assert_eq!(erc20.balance_of(accounts.alice), 100);
            // Bob does not owns tokens
            assert_eq!(erc20.balance_of(accounts.bob), 0);
        }

        #[ink::test]
        fn convert_to_shares_works() {
            let erc20 = Erc4626::new(100, 10);
            assert_eq!(erc20.convert_to_shares(100), 100);
        }

        #[ink::test]
        fn convert_to_assets_works() {
            let erc20 = Erc4626::new(100, 10);
            assert_eq!(erc20.convert_to_assets(100), 100);
        }

        #[ink::test]
        fn transfer_works() {
            // Constructor works.
            let mut erc20 = Erc4626::new(100, 10);
            // Transfer event triggered during initial construction.
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            assert_eq!(erc20.balance_of(accounts.bob), 0);
            // Alice transfers 10 tokens to Bob.
            assert_eq!(erc20.transfer(accounts.bob, 10), Ok(()));
            // Bob owns 10 tokens.
            assert_eq!(erc20.balance_of(accounts.bob), 10);

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
            // Check first transfer event related to ERC-20 instantiation.
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            // Check the second transfer event relating to the actual trasfer.
            assert_transfer_event(
                &emitted_events[1],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x02; 32])),
                10,
            );
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            // Constructor works.
            let mut erc20 = Erc4626::new(100, 10);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            assert_eq!(erc20.balance_of(accounts.bob), 0);

            // Set the contract as callee and Bob as caller.
            let contract = ink::env::account_id::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(contract);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Bob fails to transfers 10 tokens to Eve.
            assert_eq!(
                erc20.transfer(accounts.eve, 10),
                Err(ErcError::InsufficientBalance)
            );
            // Alice owns all the tokens.
            assert_eq!(erc20.balance_of(accounts.alice), 100);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.eve), 0);

            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            // Constructor works.
            let mut erc20 = Erc4626::new(100, 10);
            // Transfer event triggered during initial construction.
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10),
                Err(ErcError::InsufficientAllowance)
            );
            // Alice approves Bob for token transfers on her behalf.
            assert_eq!(erc20.approve(accounts.bob, 10), Ok(()));

            // The approve event takes place.
            assert_eq!(ink::env::test::recorded_events().count(), 2);

            // Set the contract as callee and Bob as caller.
            let contract = ink::env::account_id::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(contract);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Bob transfers tokens from Alice to Eve.
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10),
                Ok(())
            );
            // Eve owns tokens.
            assert_eq!(erc20.balance_of(accounts.eve), 10);

            // Check all transfer events that happened during the previous calls:
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            // The second event `emitted_events[1]` is an Approve event that we skip
            // checking.
            assert_transfer_event(
                &emitted_events[2],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x05; 32])),
                10,
            );
        }

        #[ink::test]
        fn allowance_must_not_change_on_failed_transfer() {
            let mut erc20 = Erc4626::new(100, 10);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Alice approves Bob for token transfers on her behalf.
            let alice_balance = erc20.balance_of(accounts.alice);
            let initial_allowance = alice_balance + 2;
            assert_eq!(erc20.approve(accounts.bob, initial_allowance), Ok(()));

            // Get contract address.
            let callee = ink::env::account_id::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(callee);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Bob tries to transfer tokens from Alice to Eve.
            let emitted_events_before = ink::env::test::recorded_events().count();
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, alice_balance + 1),
                Err(ErcError::InsufficientBalance)
            );
            // Allowance must have stayed the same
            assert_eq!(
                erc20.allowance(accounts.alice, accounts.bob),
                initial_allowance
            );
            // No more events must have been emitted
            assert_eq!(
                emitted_events_before,
                ink::env::test::recorded_events().count()
            )
        }

        /// For calculating the event topic hash.
        struct PrefixedValue<'a, 'b, T> {
            pub prefix: &'a [u8],
            pub value: &'b T,
        }

        impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
        {
            #[inline]
            fn size_hint(&self) -> usize {
                self.prefix.size_hint() + self.value.size_hint()
            }

            #[inline]
            fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
                self.prefix.encode_to(dest);
                self.value.encode_to(dest);
            }
        }

        fn encoded_into_hash<T>(entity: &T) -> Hash
        where
            T: scale::Encode,
        {
            use ink::{
                env::hash::{Blake2x256, CryptoHash, HashOutput},
                primitives::Clear,
            };

            let mut result = Hash::CLEAR_HASH;
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result;
            }
            let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::ErcError::ErcError>>;

        #[ink_e2e::test]
        async fn e2e_transfer(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let total_supply = 1_000_000_000;
            let constructor = Erc20Ref::new(total_supply);
            let contract_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let total_supply_msg = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.total_supply());
            let total_supply_res = client
                .call_dry_run(&ink_e2e::bob(), &total_supply_msg, 0, None)
                .await;

            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let transfer_to_bob = 500_000_000u128;
            let transfer = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.transfer(bob_account.clone(), transfer_to_bob));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            let balance_of = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;

            // then
            assert_eq!(
                total_supply,
                total_supply_res.return_value(),
                "total_supply"
            );
            assert_eq!(transfer_to_bob, balance_of_res.return_value(), "balance_of");

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_allowances(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let total_supply = 1_000_000_000;
            let constructor = Erc20Ref::new(total_supply);
            let contract_acc_id = client
                .instantiate("erc20", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when

            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let charlie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie);

            let amount = 500_000_000u128;
            let transfer_from = build_message::<Erc20Ref>(contract_acc_id.clone()).call(|erc20| {
                erc20.transfer_from(bob_account.clone(), charlie_account.clone(), amount)
            });
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), transfer_from, 0, None)
                .await;

            assert!(
                transfer_from_result.is_err(),
                "unapproved transfer_from should fail"
            );

            // Bob approves Charlie to transfer up to amount on his behalf
            let approved_value = 1_000u128;
            let approve_call = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.approve(charlie_account.clone(), approved_value));
            client
                .call(&ink_e2e::bob(), approve_call, 0, None)
                .await
                .expect("approve failed");

            // `transfer_from` the approved amount
            let transfer_from = build_message::<Erc20Ref>(contract_acc_id.clone()).call(|erc20| {
                erc20.transfer_from(bob_account.clone(), charlie_account.clone(), approved_value)
            });
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), transfer_from, 0, None)
                .await;
            assert!(
                transfer_from_result.is_ok(),
                "approved transfer_from should succeed"
            );

            let balance_of = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;

            // `transfer_from` again, this time exceeding the approved amount
            let transfer_from = build_message::<Erc20Ref>(contract_acc_id.clone())
                .call(|erc20| erc20.transfer_from(bob_account.clone(), charlie_account.clone(), 1));
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), transfer_from, 0, None)
                .await;
            assert!(
                transfer_from_result.is_err(),
                "transfer_from exceeding the approved amount should fail"
            );

            assert_eq!(
                total_supply - approved_value,
                balance_of_res.return_value(),
                "balance_of"
            );

            Ok(())
        }
    }
}

#[derive(scale::Encode, scale::Decode)]pub enum RuntimeCall {
    /// This index can be found by investigating runtime configuration. You can check the
    /// pallet order inside `construct_runtime!` block and read the position of your
    /// pallet (0-based).
    ///
    /// https://github.com/zeitgeistpm/zeitgeist/blob/3d9bbff91219bb324f047427224ee318061a6d43/runtime/common/src/lib.rs#L254-L363
    ///
    /// [See here for more.](https://substrate.stackexchange.com/questions/778/how-to-get-pallet-index-u8-of-a-pallet-in-runtime)
    #[codec(index = 40)]
    AssetManager(AssetManagerCall),
}

#[derive(scale::Encode, scale::Decode, )]
pub enum AssetManagerCall {
    // https://github.com/open-web3-stack/open-runtime-module-library/blob/22a4f7b7d1066c1a138222f4546d527d32aa4047/currencies/src/lib.rs#L129-L131C19
    #[codec(index = 0)]
    Transfer {
        dest: MultiAddress<AccountId, ()>,
        currency_id: ZeitgeistAsset,
        #[codec(compact)]
        amount: u128,
    },
}

#[derive(Debug, Clone, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub enum ZeitgeistAsset {
    CategoricalOutcome, //(MI, CategoryIndex),
    ScalarOutcome,      //(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare, //(SerdeWrapper<PoolId>),
    Ztg,       // default
    ForeignAsset(u32),
}
