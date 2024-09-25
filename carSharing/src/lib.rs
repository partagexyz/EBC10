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
}

#[blueprint]
mod car_sharing {
    struct carSharing {
        // The vault will hold car NFTs
        cars_vault: Vault,
        available_cars: BTreeMap<String, (Global<ResourceAddress>, NonFungibleId)>,
    }

    impl carSharing {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        pub fn instantiate_car_sharing() -> Global<carSharing> {
            // Create a new resource called "CarNFT" for non-fungible car tokens
            let car_nft: Bucket = ResourceBuilder::new_ruid_non_fungible::<Car>(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "CarNFT".to_string(), locked;
                        "symbol" => "CAR".to_string(), locked;
                    }
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
                        },
                        Car {
                            make: "BMW".to_string(),
                            model: "i3".to_string(),
                            year: 2021,
                            vin: "WBY7Z4C57M7XXXXX".to_string(),
                            mileage: 18300,
                            location: "Lisboa".to_string(),
                        },
                        Car {
                            make: "Audi".to_string(),
                            model: "e-tron".to_string(),
                            year: 2023,
                            vin: "WA1LAAGE4LB0XXXXX".to_string(),
                            mileage: 20000,
                            location: "Barcelona".to_string(),
                        },
                        Car {
                            make: "Nissan".to_string(),
                            model: "Leaf".to_string(),
                            year: 2020,
                            vin: "1N4AZ1CP7LC3XXXXX".to_string(),
                            mileage: 23000,
                            location: "Paris".to_string(),
                        },
                        Car {
                            make: "Chevrolet".to_string(),
                            model: "Bolt".to_string(),
                            year: 2021,
                            vin: "1G1FY6S00M4XXXXX".to_string(),
                            mileage: 12000,
                            location: "London".to_string(),
                        },
                    ],
                )
                .into();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                cars_vault: Vault::with_bucket(car_nft),
                available_cars: BTreeMap::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        // Method to add a new car to the platform
        pub fn add_car(&mut self, car: Car) -> Bucket {
            let new_car_bucket = self.cars_vault.authorize(|| {
                ResourceBuilder::new_ruid_non_fungible::<Car>(OwnerRole::None)
                    .mint(vec![car])
            });

            // Assuming each car has a unique ID based on make, model, and year
            let car_id = format!("{}_{}_{}", car.make, car.model, car.year);
            self.available_cars.insert(car_id, (new_car_bucket.resource_address(), new_car_bucket.non_fungible().id()));

            new_car_bucket
        }

        // Method to allow users to rent a car (retrieve a car NFT)
        pub fn rent_car(&mut self, car_id: String) -> Bucket {
            if let Some((resource, non_fungible_id)) = self.available_cars.remove(&car_id) {
                // Remove the car from the vault
                self.cars_vault.take_non_fungible(&non_fungible_id)
            } else {
                // Handle car not found
                panic!("Car not found or already rented")
            }
        }

        // Method to return a car
        pub fn return_car(&mut self, car_bucket: Bucket) {
            // Assuming the bucket contains exactly one car
            let car_non_fungible = car_bucket.non_fungible();
            let car_id = format!("{}_{}_{}", 
                car_non_fungible.data().make, 
                car_non_fungible.data().model, 
                car_non_fungible.data().year
            );
            self.available_cars.insert(car_id, (car_bucket.resource_address(), car_non_fungible.id()));
            self.cars_vault.put(car_bucket);
        }

        // Method to list available cars
        pub fn list_available_cars(&self) -> Vec<String> {
            self.available_cars.keys().cloned().collect()
        }
    }
}
