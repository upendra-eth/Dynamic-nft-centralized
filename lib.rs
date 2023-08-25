#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
mod PeerNftcontract {

    use ink::{
        codegen::{EmitEvent, Env},
    };

    use openbrush::{
        contracts::{
            access_control::{extensions::enumerable::*, only_role},
            psp34::{
                extensions::{
                    burnable::*,
                    enumerable::*,
                    metadata::{self, *},
                },
                Id, PSP34Error,
            },
        },
        storage::Mapping,
        traits::{DefaultEnv, Storage, String},
    };

    use ink::prelude::vec::Vec;
    use openbrush::contracts::psp34::balances::BalancesManager;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct PeerNftcontract {
        #[storage_field]
        psp34: psp34::Data<Balances>,
        #[storage_field]
        access: access_control::Data<Members>,
        #[storage_field]
        metadata: metadata::Data,

        // Fields of current contract
        /// mapping from token id to `token_uri`
        token_uris: Mapping<Id, String>,

        /// mapping from token id to `token_locations`
        token_locations: Mapping<Id, String>,

        /// A unique identifier for the tokens which have been minted (and are therefore
        /// supported) by this contract.
        next_id: u32,
    }

    const MANAGER: RoleType = ink::selector_id!("MANAGER");
    // const MINTER: RoleType = ink::selector_id!("MINTER");
    // const BURNER: RoleType = ink::selector_id!("BURNER");

    // Section contains default implementation without any modifications
    impl PSP34 for PeerNftcontract {}
    impl AccessControl for PeerNftcontract {}
    impl AccessControlEnumerable for PeerNftcontract {}
    impl PSP34Enumerable for PeerNftcontract {}
    impl PSP34Metadata for PeerNftcontract {}

    impl PSP34Burnable for PeerNftcontract {
        #[ink(message)]
        fn burn(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            let owner = self.owner_of(id.clone()).unwrap();
            let caller = Self::env().caller();

            if owner != caller && !self._allowance(&owner, &caller, &Some(&id)) {
                return Err(PSP34Error::NotApproved);
            }
            self.remove_token_uri(id.clone());
            self.remove_token_location(id.clone());
            self._burn_from(account, id)
        }
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: Option<Id>,
        approved: bool,
    }

    /// Event emitted when a set_token_uri occurs.
    #[ink(event)]
    pub struct SetTokenUri {
        #[ink(topic)]
        _id: Id,
        #[ink(topic)]
        _token_uri: String,
    }

    /// Event emitted when a set_token_location occurs.
    #[ink(event)]
    pub struct SetTokenLocation {
        #[ink(topic)]
        _id: Id,
        #[ink(topic)]
        _token_location: String,
    }

    /// Event emitted when a attribute_set occurs.
    #[ink(event)]
    pub struct SetAttribute {
        #[ink(topic)]
        _id: Id,
        #[ink(topic)]
        _key: String,
        #[ink(topic)]
        _data: String,
    }

    /// Event emitted when a role admin changed occurs.
    #[ink(event)]
    pub struct RoleAdminChanged {
        #[ink(topic)]
        _role: RoleType,
        #[ink(topic)]
        _previous: RoleType,
        #[ink(topic)]
        _new: RoleType,
    }

    /// Event emitted when a role grant occurs.
    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        _role: RoleType,
        #[ink(topic)]
        _grantee: AccountId,
        #[ink(topic)]
        _grantor:Option<AccountId>,
    }

    /// Event emitted when a role revoked occurs.
    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        _role: RoleType,
        #[ink(topic)]
        _account: AccountId,
        #[ink(topic)]
        _sender: AccountId,
    }

    // Override event emission methods
    impl access_control::Internal for PeerNftcontract {
        default fn _emit_role_admin_changed(
            &mut self,
            _role: RoleType,
            _previous: RoleType,
            _new: RoleType,
        ) {
            self.env().emit_event(RoleAdminChanged {
                _role,
                _previous,
                _new,
            });
        }
        default fn _emit_role_granted(
            &mut self,
            _role: RoleType,
            _grantee: AccountId,
            _grantor: Option<AccountId>,
        ) {
            self.env().emit_event(RoleGranted {
                _role,
                _grantee,
                _grantor,
            });
        }
        default fn _emit_role_revoked(
            &mut self,
            _role: RoleType,
            _account: AccountId,
            _sender: AccountId,
        ) {
            self.env().emit_event(RoleRevoked {
                _role,
                _account,
                _sender,
            });
        }
    }

    // Override event emission methods
    impl psp34::Internal for PeerNftcontract {
        fn _emit_transfer_event(&self, from: Option<AccountId>, to: Option<AccountId>, id: Id) {
            self.env().emit_event(Transfer { from, to, id });
        }

        fn _emit_approval_event(
            &self,
            from: AccountId,
            to: AccountId,
            id: Option<Id>,
            approved: bool,
        ) {
            self.env().emit_event(Approval {
                from,
                to,
                id,
                approved,
            });
        }
    }

    // Override event emission methods
    impl metadata::Internal for PeerNftcontract {
        /// Event is emitted when an attribute is set for a token.
        default fn _emit_attribute_set_event(&self, _id: Id, _key: String, _data: String) {
            self.env().emit_event(SetAttribute { _id, _key, _data });
        }
    }

    impl PeerNftcontract {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(collection_name: String, collection_symbol: String) -> Self {
            let mut _instance = Self::default();
            _instance._init_with_admin(_instance.env().caller());
            _instance
                .grant_role(MANAGER, _instance.env().caller())
                .expect("Should grant MANAGER role");

            // _instance
            //     ._mint_to(_instance.env().caller(), Id::U8(1))
            //     .expect("Can mint");

            let collection_id = _instance.collection_id();
            _instance._set_attribute(collection_id.clone(), String::from("name"), collection_name);
            _instance._set_attribute(collection_id, String::from("symbol"), collection_symbol);
            _instance
        }

        pub fn _emit_set_token_uri_event(&self, _id: Id, _token_uri: String) {
            self.env().emit_event(SetTokenUri { _id, _token_uri });
        }
        pub fn _emit_updated_token_uri_event(&self, _id: Id, _token_uri: String) {
            self.env().emit_event(SetTokenUri { _id, _token_uri });
        }
        pub fn _emit_remove_token_uri_event(&self, _id: Id) {}

        /// Only manager
        // #[openbrush::modifiers(only_role(MANAGER))]
        fn set_token_uri(&mut self, id: Id, _token_uri: String) -> Result<(), PSP34Error> {
            self.token_uris.insert(&id, &_token_uri);
            self._emit_set_token_uri_event(id, _token_uri);

            Ok(())
        }

        #[ink(message)]
        pub fn get_token_uri(&self, id: Id) -> Option<String> {
            self.token_uris.get(&id)
        }

        #[ink(message)]
        pub fn manager_role_bytes(&self) -> RoleType {
            MANAGER
        }

        /// Only manager
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn remove_token_uri(&mut self, id: Id) -> Result<(), PSP34Error> {
            self.token_uris.remove(&id);
            self._emit_remove_token_uri_event(id);

            Ok(())
        }

        /// Only manager
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn update_token_uri(&mut self, id: Id, _token_uri: String) -> Result<(), PSP34Error> {
            self.token_uris.remove(&id);
            self.token_uris.insert(&id, &_token_uri);
            self._emit_updated_token_uri_event(id, _token_uri);

            Ok(())
        }

        pub fn _emit_set_token_location_event(&self, _id: Id, _token_location: String) {
            self.env().emit_event(SetTokenLocation {
                _id,
                _token_location,
            });
        }
        pub fn _emit_updated_token_location_event(&self, _id: Id, _token_location: String) {
            self.env().emit_event(SetTokenLocation {
                _id,
                _token_location,
            });
        }
        pub fn _emit_remove_token_location_event(&self, _id: Id) {}

        /// Only manager
        // #[openbrush::modifiers(only_role(MANAGER))]
        fn set_token_location(
            &mut self,
            id: Id,
            _token_location: String,
        ) -> Result<(), PSP34Error> {
            self.token_locations.insert(&id, &_token_location);
            self._emit_set_token_location_event(id, _token_location);

            Ok(())
        }

        #[ink(message)]
        pub fn get_token_location(&self, id: Id) -> Option<String> {
            self.token_locations.get(&id)
        }

        /// Only manager
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn remove_token_location(&mut self, id: Id) -> Result<(), PSP34Error> {
            self.token_locations.remove(&id);
            self._emit_remove_token_location_event(id);

            Ok(())
        }

        /// Only manager
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn update_token_location(
            &mut self,
            id: Id,
            _token_location: String,
        ) -> Result<(), PSP34Error> {
            self.token_locations.remove(&id);
            self.token_locations.insert(&id, &_token_location);
            self._emit_updated_token_location_event(id, _token_location);

            Ok(())
        }

        #[ink(message)]
        pub fn mint(
            &mut self,
            account: AccountId,
            _token_location: String,
            _token_uri: String,
        ) -> Result<(), PSP34Error> {
            self.set_token_uri(Id::U32(self.next_id), _token_uri);
            self.set_token_location(Id::U32(self.next_id), _token_location);
            self._mint_to(account, Id::U32(self.next_id));
            self.next_id += 1;
            Ok(())
        }

        /// Only manager
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn manager_mint(
            &mut self,
            account: AccountId,
            _token_location: String,
            _token_uri: String,
        ) -> Result<(), PSP34Error> {
            self.set_token_uri(Id::U32(self.next_id), _token_uri);
            self.set_token_location(Id::U32(self.next_id), _token_location);
            self._mint_to(account, Id::U32(self.next_id));
            self.next_id += 1;
            Ok(())
        }

        /// Only manager
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn manager_burn(
            &mut self,
            account: AccountId,
            id: Id,
            _token_location: String,
            _token_uri: String,
        ) -> Result<(), PSP34Error> {
            self.remove_token_uri(id.clone());
            self.remove_token_location(id.clone());
            self._burn_from(account, id);
            Ok(())
        }

        /// Only manager
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn manager_transfer(
            &mut self,
            to: AccountId,
            id: Id,
            _data: Vec<u8>,
        ) -> Result<(), PSP34Error> {
            self._transfer(to, id, _data)
        }

        /// Internal transfer function
        fn _transfer(&mut self, to: AccountId, id: Id, _data: Vec<u8>) -> Result<(), PSP34Error> {
            let owner = self._check_token_exists(&id)?;
            let caller = self.env().caller();

            self._before_token_transfer(Some(&owner), Some(&to), &id)?;

            self.psp34
                .operator_approvals
                .remove(&(&owner, &caller, &Some(&id)));
            self.psp34.balances.decrease_balance(&owner, &id, false);
            self.psp34.token_owner.remove(&id);

            self.psp34.balances.increase_balance(&to, &id, false);
            self.psp34.token_owner.insert(&id, &to);
            self._after_token_transfer(Some(&owner), Some(&to), &id)?;
            self._emit_transfer_event(Some(owner), Some(to), id);

            Ok(())
        }

        /// Modifies the code which is used to execute calls to this contract address (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here, any caller
        /// can execute this method. In a production contract you would do some authorization here.
        #[ink(message)]
        #[openbrush::modifiers(only_role(MANAGER))]
        pub fn set_code(&mut self, code_hash: [u8; 32]) -> Result<(), PSP34Error> {
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });
            Ok(())
        }

        #[ink(message)]
        pub fn nft_ids_of(&self, owner: AccountId) -> Option<Vec<u32>> {
            let mut ids_vec: Vec<u32> = Vec::new();
            for n in 0..self.next_id {
                    let ids = Id::U32(n);
                    let addr = self.owner_of(ids.clone()).unwrap();
                    if owner == addr {
                        ids_vec.push(n);
                }
            }
            return Some(ids_vec);
        }
    }

    ////////////////////////////////////////////// Off Chain Tests /////////////////////////////////////////

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink::codegen::Env;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn constructor_works_name() {
            let PeerNftcontract =
                PeerNftcontract::new("My First NFT".into(), "Nova".into());
            let collection_id = PeerNftcontract.collection_id();
            let key = String::from("name");

            assert_eq!(
                PeerNftcontract
                    .get_attribute(collection_id, key)
                    .unwrap(),
                b"My First NFT"
            );
        }

        #[ink::test]
        fn constructor_works_symbol() {
            let PeerNftcontract =
                PeerNftcontract::new("My First NFT".into(), "Nova".into());
            let collection_id = PeerNftcontract.collection_id();
            let key = String::from("symbol");
            assert_eq!(
                PeerNftcontract
                    .get_attribute(collection_id, key)
                    .unwrap(),
                "Nova".as_bytes()
            );
        }

        #[ink::test]
        fn check_admin_role() {
            let PeerNftcontract =
                PeerNftcontract::new("My First NFT".into(), "Nova".into());
            assert_eq!(PeerNftcontract.get_role_admin(0), 0);
        }

        #[ink::test]
        fn check_what_admin_role_contains() {
            let PeerNftcontract =
                PeerNftcontract::new("My First NFT".into(), "Nova".into());
            assert_eq!(PeerNftcontract.access.admin_roles.get(0), None);
        }

        #[ink::test]
        fn check_do_admin_role_contains() {
            let PeerNftcontract =
                PeerNftcontract::new("My First NFT".into(), "Nova".into());
            assert_eq!(PeerNftcontract.access.admin_roles.contains(0), false);
        }

        // #[ink::test]
        // fn check_role_admin() {
        //     let PeerNftcontract =
        //         PeerNftcontract::new("My First NFT".into(), "Nova".into());
        //     let caller = PeerNftcontract.env().caller();
        //     assert_eq!(
        //         PeerNftcontract
        //             .access
        //             .members
        //             .members
        //             .contains(&(0u32, &caller)),
        //         true
        //     );
        // }



//         #[ink::test]
//         fn check_only_role_modifier() {
//             let mut PeerNftcontract =
//                 PeerNftcontract::new("My First NFT".into(), "Nova".into());

//             let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//             ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//             let burner_role =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER".to_string());
//             let burner_role2 =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER2".to_string());
//             let caller = PeerNftcontract.env().caller();

//             PeerNftcontract.add_to_members_map_by_setup_role(burner_role, accounts.bob);

//             let _ = PeerNftcontract.mint(accounts.bob, Id::U8(2u8));
//             assert_eq!(
//                 PeerNftcontract.owner_of(Id::U8(2u8)).unwrap(),
//                 accounts.bob
//             );
//             //  assert_eq!(PeerNftcontract.owner_of(Id::U8(2u8)),None );
//         }

//         #[ink::test]
//         fn check_grant_role() {
//             let mut PeerNftcontract =
//                 PeerNftcontract::new("My First NFT".into(), "Nova".into());

//             let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

//             // Set the Bob as caller.
//             ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

//             let burner_admin_role =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER-ADMIN".to_string());
//             let burner_role =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER".to_string());
//             let burner_role2 =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER2".to_string());
//             let caller = PeerNftcontract.env().caller();

//             PeerNftcontract.put_in_admin_role_map_indirectly(burner_role, burner_admin_role);
//             PeerNftcontract.add_to_members_map_by_setup_role(burner_admin_role, caller);
//             assert_eq!(
//                 PeerNftcontract.grant_role(burner_role, accounts.alice),
//                 Ok(())
//             );

//             //  `alice`, `bob`, `charlie`, `django`, `eve`, `frank`
//             let _ = PeerNftcontract.grant_role(burner_role, accounts.charlie);
//             assert_eq!(
//                 PeerNftcontract.has_role(burner_role, accounts.charlie),
//                 true
//             );
//             // Set the charlie as caller.
//             //  ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

//             let _ = PeerNftcontract.grant_role(burner_role, accounts.django);
//             assert_eq!(
//                 PeerNftcontract.has_role(burner_role, accounts.django),
//                 true
//             )
//         }

//         #[ink::test]
//         fn check_mint() {
//             let mut PeerNftcontract =
//                 PeerNftcontract::new("My First NFT".into(), "Nova".into());

//             let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

//             // Set the Bob as caller.
//             ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

//             let burner_admin_role =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER-ADMIN".to_string());
//             let burner_role =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER".to_string());
//             let burner_role2 =
//                 PeerNftcontract.role_name_string_to_roletype("BURNER2".to_string());
//             let caller = PeerNftcontract.env().caller();

//             PeerNftcontract.put_in_admin_role_map_indirectly(burner_role, burner_admin_role);
//             PeerNftcontract.add_to_members_map_by_setup_role(burner_admin_role, caller);

//             let _ = PeerNftcontract.grant_role(burner_role, accounts.bob);
//             assert_eq!(
//                 PeerNftcontract.has_role(burner_role, accounts.bob),
//                 true
//             );

//             let _ = PeerNftcontract.mint(accounts.alice, Id::U8(2u8));
//             assert_eq!(
//                 PeerNftcontract.owner_of(Id::U8(2u8)).unwrap(),
//                 accounts.alice
//             );
//             // assert_eq!(PeerNftcontract.owner_of(Id::U8(2u8)),None );
//         }
    }
}
