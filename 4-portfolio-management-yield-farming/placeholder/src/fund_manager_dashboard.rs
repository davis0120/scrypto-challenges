use scrypto::prelude::*;
use crate::index_fund::*;
use crate::fundinglocker::*;
use crate::lending_pool::*;
use crate::maple_finance_global::*;
use crate::structs::*;
use crate::utils::*;

// Allows approved Fund Manager to manage pools.

blueprint! {
    struct FundManagerDashboard {
        fund_manager_address: ResourceAddress,
        fund_manager_id: NonFungibleId,
        fund_manager_admin_vault: Vault,
        lending_pools: HashMap<(String, ResourceAddress), ComponentAddress>,
        funding_lockers: HashMap<NonFungibleId, ComponentAddress>,
        loan_request_nft_admin: Vault,
        loan_nft_admin: Vault,
        loan_nft_address: ResourceAddress,
        fund_master_admin: Vault,
        price_oracle_address: ComponentAddress,
        maple_finance_global_address: ComponentAddress,
    }

    impl FundManagerDashboard {

        pub fn new(
            fund_manager_admin: Bucket,
            maple_finance_global_address: ComponentAddress,
            fund_manager_address: ResourceAddress,
            fund_manager_id: NonFungibleId,
            loan_request_nft_admin: Bucket,
            price_oracle_address: ComponentAddress,
        ) -> ComponentAddress
        {
            let fund_master_admin = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Fund Manager Master Admin Badge")
                .metadata("symbol", "PDMAB")
                .metadata("description", "Allows Fund Managers to mint/burn loan NFTs.")
                .initial_supply(1);

            let loan_nft_admin = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Loan NFT Admin Badge")
                .metadata("symbol", "LNFTAB")
                .metadata("description", "Allows Fund Managers to mint/burn loan NFTs.")
                .mintable(rule!(require(fund_master_admin.resource_address())), LOCKED)
                .burnable(rule!(require(fund_master_admin.resource_address())), LOCKED)
                .initial_supply(1);

            let loan_nft_address: ResourceAddress = ResourceBuilder::new_non_fungible()
                .metadata("name", "Loan NFT")
                .metadata("symbol", "LNFT")
                .metadata("description", "Loan NFT")
                .mintable(rule!(require(loan_nft_admin.resource_address())), LOCKED)
                .burnable(rule!(require(loan_nft_admin.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(loan_nft_admin.resource_address())), LOCKED)
                .no_initial_supply();

            return Self {
                fund_manager_admin_vault: Vault::with_bucket(fund_manager_admin),
                lending_pools: HashMap::new(),
                funding_lockers: HashMap::new(),
                fund_manager_address: fund_manager_address,
                fund_manager_id: fund_manager_id,
                loan_request_nft_admin: Vault::with_bucket(loan_request_nft_admin),
                loan_nft_admin: Vault::with_bucket(loan_nft_admin),
                loan_nft_address: loan_nft_address,
                fund_master_admin: Vault::with_bucket(fund_master_admin),
                price_oracle_address: price_oracle_address,
                maple_finance_global_address: maple_finance_global_address,
            }
            .instantiate()
            .globalize();
        }

        /// Checks if a lending pool for the given token exists or not.
        pub fn pool_exists(
            &self,
            pool_name: String,
            address: ResourceAddress) -> bool
        {
            return self.lending_pools.contains_key(&(pool_name, address));
        }

        /// Asserts that a lending pool for the given address exists
        pub fn assert_pool_exists(
            &self,
            pool_name: String,
            address: ResourceAddress,
            label: String) 
        {
            assert!(
                self.pool_exists(pool_name, address), 
                "[{}]: No lending pool exists for the given address.", 
                label
            );
        }
        
        /// Asserts that a lending pool for the given address pair doesn't exist.
        pub fn assert_pool_doesnt_exists(
            &self,
            pool_name: String, 
            address: ResourceAddress, 
            label: String) 
        {
            assert!(
                !self.pool_exists(pool_name, address), 
                "[{}]: A lending pool exists with the given address.", 
                label
            );
        }

        pub fn new_lending_pool(
            &mut self,
            fund_admin_admin_badge: Proof,
            pool_name: String,
            initial_funds: Bucket) -> (ComponentAddress, Bucket)
        {
            // Checking if a lending pool already exists for this token.
            self.assert_pool_doesnt_exists(
                pool_name.clone(), 
                initial_funds.resource_address(), 
                String::from("New Liquidity Pool")
            );

            //
            let token_address = initial_funds.resource_address();

            let fund_manager_id: NonFungibleId = fund_admin_admin_badge.non_fungible::<FundManager>().id();

            // Instantiates the lending pool and collateral pool.
            let (lending_pool, tracking_tokens): (ComponentAddress, Bucket) = LendingPool::new(
                fund_admin_admin_badge.resource_address(), 
                fund_manager_id, 
                initial_funds
            );

            self.lending_pools.insert(
                (pool_name, token_address),
                lending_pool
            );

            (lending_pool, tracking_tokens)
        }

        // pub fn retrieve_loan_requests(
        //     &self) -> HashMap<ResourceAddress, BTreeSet<NonFungibleId>>
        // {
        //     let maple_finance_global: MapleFinance = self.maple_finance_global_address.unwrap().into();
        //     let loan_requests = maple_finance_global.broadcast_loan_requests();
        //     loan_requests
        // }

        pub fn instantiate_funding_locker(
            &mut self,
            fund_admin_admin_badge: Proof,
            loan_request_nft_id: NonFungibleId,
            loan_request_nft_address: ResourceAddress,
            borrower_id: NonFungibleId,
            loan_amount: Decimal,
            asset_address: ResourceAddress,
            collateral_address: ResourceAddress,
            collateral_percent: Decimal,
            annualized_interest_rate: Decimal,
            term_length: u64,
            payment_frequency: PaymentFrequency,
            origination_fee: Decimal,
        ) 
        {
            assert_eq!(fund_admin_admin_badge.resource_address(), self.fund_manager_address,
                "[Fund Manager Dashboard]: This badge does not belong to this protocol."
            );

            assert_eq!(fund_admin_admin_badge.non_fungible::<FundManager>().id(), self.fund_manager_id,
                "[Fund Manager Dashboard]: Incorrect Fund Manager."
            );

            let origination_fee_charged = loan_amount * origination_fee;
            let annualized_interest_expense = loan_amount * annualized_interest_rate;
            let remaining_balance = loan_amount + origination_fee;

            let loan_nft = self.loan_nft_admin.authorize(|| {
                let resource_manager: &ResourceManager = borrow_resource_manager!(self.loan_nft_address);
                resource_manager.mint_non_fungible(
                    // The User id
                    &NonFungibleId::random(),
                    // The User data
                    Loan {
                        borrower_id: borrower_id,
                        lender_id: self.fund_manager_id.clone(),
                        lender_address: self.fund_manager_address,
                        principal_loan_amount: loan_amount,
                        asset: asset_address,
                        collateral: collateral_address,
                        collateral_percent: collateral_percent,
                        annualized_interest_rate: annualized_interest_rate,
                        term_length: term_length,
                        payment_frequency: payment_frequency,
                        origination_fee: origination_fee,
                        origination_fee_charged: origination_fee_charged,
                        annualized_interest_expense: annualized_interest_expense,
                        remaining_balance: remaining_balance,
                        last_update: Runtime::current_epoch(),
                        collateral_amount: Decimal::zero(),
                        collateral_amount_usd: Decimal::zero(),
                        health_factor: Decimal::zero(),
                        loan_status: Status::Current,
                    },
                )
            });

            let loan_nft_id = loan_nft.non_fungible::<Loan>().id();

            let loan_nft_admin = self.fund_master_admin.authorize(|| borrow_resource_manager!(self.loan_nft_admin.resource_address()).mint(1));

            let funding_locker: ComponentAddress = FundingLocker::new(
                loan_request_nft_id.clone(), 
                loan_request_nft_address, 
                loan_nft, 
                loan_nft_admin
            );

            self.funding_lockers.insert(
                loan_nft_id.clone(),
                funding_locker
            );

            // * MODIFIES LOAN REQUEST NFT * //
            // Retrieves resource manager for the Loan Request NFT.
            let resource_manager = borrow_resource_manager!(loan_request_nft_address);

            let mut loan_request_nft_data: LoanRequest = resource_manager.get_non_fungible_data(&loan_request_nft_id);

            loan_request_nft_data.status = RequestStatus::Modified;
            loan_request_nft_data.loan_nft_id = Some(loan_nft_id); 
            loan_request_nft_data.funding_locker_address = Some(funding_locker);
            
            self.loan_request_nft_admin.authorize(||
                resource_manager.update_non_fungible_data(&loan_request_nft_id, loan_request_nft_data)
            );
        }

        // pub fn fund_loan(
        //     &mut self,
        //     fund_admin_badge: Proof,
        //     pool_name: String,
        //     funding_amount: Decimal,
        //     funding_terms: Bucket)
        // {
        //     assert_eq!(
        //         fund_admin_badge.resource_address(), self.fund_admin_admin_address,
        //         "[Fund Manager Dashboard: Incorrect Proof passed."
        //     );

        //     assert_eq!(
        //         fund_admin_badge.non_fungible::<FundManager>().id(), self.fund_manager_id,
        //         "[Fund Manager Dashboard: Incorrect Proof passed."
        //     );

        //     let optional_lending_pool: Option<&ComponentAddress> = self.lending_pools.get(&token_requested);
        //     match optional_lending_pool {
        //         Some (lending_pool) => { // If it matches it means that the lending pool exists.
        //             lending_pool: LendingPool = optional_lending_pool.unwrap().into();
        //             let funding_locker: ComponentAddress = lending_pool.fund_loan(
        //                 fund_admin_badge,
        //                 funding_amount,
        //                 funding_terms
        //             );

        //         }
        //         None => { 
        //             info!("[DegenFi]: Pool for {:?} doesn't exist.", token_requested);

        //         }
        //     }
        // } 

        pub fn create_fund(
            &mut self,
            fund_admin_badge: Proof,
            fund_type: FundType,
            fund_name: String,
            fee_to_pool: Decimal,
            fund_ticker: String,
            starting_share_price: Decimal,
            tokens: HashMap<ResourceAddress, Decimal>,
        ) -> Bucket
        {
            let maple_finance: MapleFinance = self.maple_finance_global_address.into();

            assert_eq!(fund_admin_badge.resource_address(), self.fund_manager_address,
                "[Fund Manager Dashboard]: This badge does not belong to this protocol."
            );

            assert_eq!(fund_admin_badge.non_fungible::<FundManager>().id(), self.fund_manager_id,
                "[Fund Manager Dashboard]: Incorrect Fund Manager."
            );

            assert_ne!(maple_finance.assert_index_fund(fund_name.clone()), true, 
                "[Fund Manager Dashboard]: The name or ticker for this fund already exist. Please choose another."
            );

            let (fund_name, fund_ticker): (String, String) = sort_string(fund_name.clone(), fund_ticker);
            let fund_id: (String, String) = (fund_name.clone(), fund_ticker.clone());

            let price_oracle_address: ComponentAddress = self.price_oracle_address;

            let (index_fund, fund_admin): (ComponentAddress, Bucket) = IndexFund::new(
                fund_name.clone(), 
                fund_ticker.clone(),
                fee_to_pool, 
                starting_share_price,
                tokens,
                price_oracle_address,
            );

            let fund_manager_data: FundManager = self.get_resource_manager();
            let mut managed_index_funds = fund_manager_data.managed_index_funds;
            
            managed_index_funds.insert(fund_id.clone(), index_fund);

            self.authorize_update(fund_manager_data);

            maple_finance.insert_index_fund_name(fund_name.clone(), fund_ticker.clone());
            maple_finance.insert_index_fund(fund_id, index_fund);

            fund_admin
        }

        fn get_resource_manager(
            &self,
        ) -> FundManager
        {
            let resource_manager = borrow_resource_manager!(self.fund_manager_address);
            let fund_manager_data: FundManager = resource_manager.get_non_fungible_data(&self.fund_manager_id);

            fund_manager_data 
        }

        fn authorize_update(
            &self,
            fund_manager_data: FundManager
        )
        {
            let resource_manager = borrow_resource_manager!(self.fund_manager_address);
            self.fund_manager_admin_vault.authorize(|| 
                resource_manager.update_non_fungible_data(&self.fund_manager_id, fund_manager_data)
            );
        }
    }
}