use scrypto::prelude::*;
#[derive(ScryptoSbor, NonFungibleData)]
pub struct Car {
    // Define what data will be stored for each car NFT
    make: String,
    model: String,
    year: u32,
    vin: String, // unique Vehicle Identification Number
    mileage: u32,
    location: String,
    price_per_hour: Decimal,
    owner_badge_id: NonFungibleLocalId,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct CarOwnerBadge {
    owner_number: u64,
    owner_name: String,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct UserBadge {
    user_number: u64,
    user_name: String,
    driving_license: String,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct RentalBadge {
    car_nft_id: NonFungibleGlobalId,
    user_badge_id: NonFungibleGlobalId,
    start_time: Instant,
    duration_in_hours: u32,
}

#[blueprint]
mod car_sharing {
    // define auth rules
    enable_method_auth! {
    roles {
        car_owner => updatable_by: [OWNER];
        user => updatable_by: [OWNER];
    },
        // decide which methods are public and which are restricted to certain roles
        methods {
            mint_car_owner_badge => restrict_to: [OWNER];
            mint_user_badge => restrict_to: [OWNER];
            mint_rental_badge => restrict_to: [OWNER];
            add_car => restrict_to: [car_owner];
            withdraw_car_owner_vault => restrict_to: [car_owner];
            rent_car => restrict_to: [user];
            return_car => restrict_to: [user];
            list_available_cars => restrict_to: [user];
        }
    }
    struct carSharing {
        // The vault will hold car NFTs
        car_sharing_vault: Vault,
        car_sharing_resource_manager: ResourceManager,
        // available_cars: BTreeMap<String, (Global<ResourceAddress>, NonFungibleId)>,
        car_owner_badge_resource_manager: ResourceManager,
        user_badge_resource_manager: ResourceManager,
        rental_badge_resource_manager: ResourceManager, // New resource for rental badges
        rental_payment_vaults: HashMap<NonFungibleLocalId, Vault>, // Mapping between UserBadge NFT and XRD vaults
        car_owner_vaults: HashMap<NonFungibleLocalId, Vault>, // Mapping between CarOwnerBadge NFT and XRD vaults
    }

    impl carSharing {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        pub fn instantiate_car_sharing() -> (Global<carSharing>, Bucket) {
            // reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(carSharing::blueprint_id());

            // create an Owner Badge
            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Car Sharing Owner Badge", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // create a new Car Owner Badge resource manager
            let car_owner_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<CarOwnerBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Car Owner Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(owner_badge.resource_address()));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            let car_owner_badge_bucket: NonFungibleLocalId = car_owner_badges_manager
                .mint_non_fungible(
                    &NonFungibleLocalId::integer(1),
                    CarOwnerBadge {
                        owner_number: 1,
                        owner_name: "Test".to_string(),
                    },
                )
                .as_non_fungible()
                .non_fungible_local_id();

            // create a new User Badge resource manager
            let user_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<UserBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "User Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // create a new User Badge resource manager
            let rental_badges_manager: ResourceManager =
                ResourceBuilder::new_ruid_non_fungible::<RentalBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Rental Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // Create a new resource called "CarNFT" for non-fungible car tokens
            let cars_bucket: Bucket =
                ResourceBuilder::new_ruid_non_fungible::<Car>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "CarNFT".to_string(), locked;
                            "symbol" => "CAR".to_string(), locked;
                        }
                    })
                    .mint_roles(mint_roles! {
                        minter => rule!(require(car_owner_badges_manager.address()));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(car_owner_badges_manager.address()));
                        burner_updater => rule!(deny_all);
                    })
                    .mint_initial_supply(
                        // Define initial cars
                        [
                            Car {
                                make: "Tesla".to_string(),
                                model: "Model S".to_string(),
                                year: 2022,
                                vin: "5YJSA1E26MF1XXXXX".to_string(),
                                mileage: 15000,
                                location: "New York".to_string(),
                                price_per_hour: dec!("10"),
                                owner_badge_id: car_owner_badge_bucket.clone(),
                            },
                            Car {
                                make: "BMW".to_string(),
                                model: "i3".to_string(),
                                year: 2021,
                                vin: "WBY7Z4C57M7XXXXX".to_string(),
                                mileage: 18300,
                                location: "Lisboa".to_string(),
                                price_per_hour: dec!("8"),
                                owner_badge_id: car_owner_badge_bucket.clone(),
                            },
                            Car {
                                make: "Audi".to_string(),
                                model: "e-tron".to_string(),
                                year: 2023,
                                vin: "WA1LAAGE4LB0XXXXX".to_string(),
                                mileage: 20000,
                                location: "Barcelona".to_string(),
                                price_per_hour: dec!("12"),
                                owner_badge_id: car_owner_badge_bucket.clone(),
                            },
                            Car {
                                make: "Nissan".to_string(),
                                model: "Leaf".to_string(),
                                year: 2020,
                                vin: "1N4AZ1CP7LC3XXXXX".to_string(),
                                mileage: 23000,
                                location: "Paris".to_string(),
                                price_per_hour: dec!("4"),
                                owner_badge_id: car_owner_badge_bucket.clone(),
                            },
                            Car {
                                make: "Chevrolet".to_string(),
                                model: "Bolt".to_string(),
                                year: 2021,
                                vin: "1G1FY6S00M4XXXXX".to_string(),
                                mileage: 12000,
                                location: "London".to_string(),
                                price_per_hour: dec!("5"),
                                owner_badge_id: car_owner_badge_bucket.clone(),
                            },
                        ],
                    )
                    .into();

            let car_rs_manager: ResourceManager = cars_bucket.resource_manager();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let car_sharing_impl: Global<carSharing> = Self {
                car_sharing_vault: Vault::with_bucket(cars_bucket),
                car_sharing_resource_manager: car_rs_manager,
                car_owner_badge_resource_manager: car_owner_badges_manager,
                user_badge_resource_manager: user_badges_manager,
                rental_badge_resource_manager: rental_badges_manager,
                rental_payment_vaults: HashMap::new(), // Empty map to store a payment vault for each RentalBadge
                car_owner_vaults: HashMap::new(), // Empty map to store a payment vault for each CarOwnerBadge
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles!(
                car_owner => rule!(require(car_owner_badges_manager.address()));
                user => rule!(require(user_badges_manager.address()));))
            .with_address(address_reservation)
            .globalize();
            (car_sharing_impl, owner_badge)
        }

        pub fn mint_car_owner_badge(&mut self, name: String, number: u64) -> Bucket {
            // mint and receive a new car owner badge. requires an owner badge
            let car_owner_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    CarOwnerBadge {
                        owner_number: number,
                        owner_name: name,
                    },
                );
            self.car_owner_vaults.insert(
                car_owner_badge_bucket
                    .as_non_fungible()
                    .non_fungible_local_id(),
                Vault::new(XRD),
            );
            car_owner_badge_bucket
        }
        pub fn mint_user_badge(&mut self, name: String, number: u64) -> Bucket {
            // mint and receive a new car owner badge. requires an owner badge
            let user_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    UserBadge {
                        user_number: number,
                        user_name: name,
                        driving_license: "TODO".to_string(),
                    },
                );
            user_badge_bucket
        }

        pub fn mint_rental_badge(
            &mut self,
            car_nft_id: NonFungibleGlobalId,
            user_badge_id: NonFungibleGlobalId,
            duration_in_hours: u32,
        ) -> Bucket {
            // TODO: why not NonFungibleBucket

            let time: Instant = Clock::current_time(TimePrecisionV2::Second);
            // Create a badge that grants temporary access to a specific car
            self.rental_badge_resource_manager
                .mint_ruid_non_fungible(RentalBadge {
                    car_nft_id: car_nft_id,
                    user_badge_id: user_badge_id,
                    start_time: time,
                    duration_in_hours: duration_in_hours,
                })
        }

        // Method to add a new car to the platform
        pub fn add_car(&mut self, car: Car) {
            self.car_sharing_vault.put(
                self.car_sharing_resource_manager
                    .mint_ruid_non_fungible(car),
            );
        }

        // Withdraw the content of a CarOwner vaul
        pub fn withdraw_car_owner_vault(&mut self, car_owner_proof: Proof) -> Bucket {
            let checked_proof: CheckedProof =
                car_owner_proof.check(self.car_owner_badge_resource_manager.address());
            let car_owner_local_id: NonFungibleLocalId =
                checked_proof.as_non_fungible().non_fungible_local_id();
            let car_owner_vault: Option<&mut Vault> =
                self.car_owner_vaults.get_mut(&car_owner_local_id);
            match car_owner_vault {
                Some(payment) => payment.take_all(),
                None => panic!("The owner vault does not exists."),
            }
        }

        // Method to allow users to rent a car (retrieve a car NFT)
        pub fn rent_car(
            &mut self,
            car_nft_id: NonFungibleGlobalId,
            user_badge_id: NonFungibleGlobalId,
            duration_in_hours: u32,
            payment_bucket: Bucket,
        ) -> (NonFungibleBucket, Bucket) {
            let xrd_vault: Vault = Vault::new(XRD);
            assert_eq!(
                xrd_vault.resource_address(),
                payment_bucket.resource_address(),
                "We only accept XRD for car rental."
            );

            let (requested_car_address, requested_car_local_id): (
                ResourceAddress,
                NonFungibleLocalId,
            ) = car_nft_id.clone().into_parts();
            assert_eq!(
                requested_car_address,
                self.car_sharing_resource_manager.address()
            );
            let rented_car: NonFungibleBucket = self
                .car_sharing_vault
                .as_non_fungible()
                .take_non_fungible(&requested_car_local_id);

            let rental_price: Decimal = self
                .car_sharing_resource_manager
                .get_non_fungible_data::<Car>(&requested_car_local_id)
                .price_per_hour
                * duration_in_hours;
            assert!(
                payment_bucket.amount() >= rental_price,
                "You did not provided enough XRD for this rental."
            );
            let rental_badge: Bucket =
                self.mint_rental_badge(car_nft_id, user_badge_id, duration_in_hours);

            let rental_payment_vault: Vault = Vault::with_bucket(payment_bucket);
            self.rental_payment_vaults.insert(
                rental_badge.as_non_fungible().non_fungible_local_id(),
                rental_payment_vault,
            );
            (rented_car, rental_badge)
        }

        // Method to return a car
        pub fn return_car(&mut self, rented_car: Bucket, rental_badge: Bucket) {
            let car_owner_badge_id: NonFungibleLocalId = self
                .car_sharing_resource_manager
                .get_non_fungible_data::<Car>(&rented_car.as_non_fungible().non_fungible_local_id())
                .owner_badge_id;
            self.car_sharing_vault.put(rented_car);
            let payment_vault: Option<Vault> = self
                .rental_payment_vaults
                .remove(&rental_badge.as_non_fungible().non_fungible_local_id());
            let payment_bucket = match payment_vault {
                Some(mut payment) => payment.take_all(),
                None => panic!("No vault associated with RentalBadge found"),
            };
            let car_owner_vault: Option<&mut Vault> =
                self.car_owner_vaults.get_mut(&car_owner_badge_id);
            match car_owner_vault {
                Some(owner_vault) => owner_vault.put(payment_bucket),
                None => panic!("The owner vault does not exists."),
            };

            rental_badge.burn();
        }

        // Method to list available cars
        pub fn list_available_cars(&self) -> indexmap::IndexSet<NonFungibleLocalId> {
            let available_cars_ids: indexmap::IndexSet<NonFungibleLocalId> = self
                .car_sharing_vault
                .as_non_fungible()
                .non_fungible_local_ids(100);
            // info!(available_cars_ids.to_string());
            available_cars_ids
        }
    }
}
