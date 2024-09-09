use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Error, ErrorKind};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use pebblevault::Vault;
use log::{error, info};
use chrono::Local;

/// Structure representing an Ingredient with required quantity and recipe crafting flag.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub recipe_craftable: bool,
}

/// Structure representing a Crafter.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Crafter {
    pub name: String,
}

/// Structure representing a Recipe.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipe {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub outcome: String,
    pub crafters: Vec<Crafter>,
    pub base_cook_time: u32,
    pub cook_count: u32,
}

impl Recipe {
    fn increment_cook_count(&mut self) {
        self.cook_count += 1;
    }

    fn is_mastered(&self) -> bool {
        self.cook_count >= 10
    }
}

/// Structure representing an Item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub model: Option<String>,
    pub meta_tags: HashMap<String, Value>,
}

/// Struct representing a Player Inventory.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerInventory {
    pub slots: HashMap<u32, Option<Item>>,
}

impl PlayerInventory {
    pub fn new(num_slots: u32) -> Self {
        let mut slots = HashMap::new();
        for i in 0..num_slots {
            slots.insert(i, None);
        }
        Self { slots }
    }

    pub fn get_item(&self, slot: u32) -> Option<&Item> {
        self.slots.get(&slot).and_then(|item| item.as_ref())
    }

    pub fn add_item(&mut self, slot: u32, item: Item) {
        self.slots.insert(slot, Some(item));
    }

    pub fn remove_item(&mut self, slot: u32) -> Option<Item> {
        self.slots.insert(slot, None).flatten()
    }

    pub fn empty_slot(&mut self, slot: u32) {
        self.slots.insert(slot, None);
    }
}

/// Struct representing a Storage Container.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageContainer {
    pub uuid: Uuid,
    pub inventory: PlayerInventory,
}

impl StorageContainer {
    pub fn new(num_slots: u32) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            inventory: PlayerInventory::new(num_slots),
        }
    }
}

/// Struct representing a Recipe Book.
pub struct RecipeBook {
    pub recipes: HashMap<String, Recipe>,
    pub crafters: HashMap<Crafter, Vec<String>>,
    pub vault: Vault,
}

impl RecipeBook {
    /// Creates a new RecipeBook.
    pub fn new() -> Self {
        let vault = Vault::new();
        vault.define_class("recipe", r#"{
            "name": "string",
            "ingredients": "array",
            "outcome": "string",
            "base_cook_time": "u32",
            "cook_count": "u32",
            "crafters": "array"
        }"#);

        vault.define_class("skill", r#"{
            "name": "string",
            "level": "u32",
            "experience": "u32"
        }"#);

        vault.define_class("achievement", r#"{
            "name": "string",
            "unlocked": "bool"
        }"#);

        Self {
            recipes: HashMap::new(),
            crafters: HashMap::new(),
            vault,
        }
    }

    /// Adds a new recipe to the RecipeBook.
    pub fn add_recipe(&mut self, recipe: Recipe) {
        for crafter in &recipe.crafters {
            self.crafters.entry(crafter.clone()).or_insert_with(Vec::new).push(recipe.name.clone());
        }
        self.recipes.insert(recipe.name.clone(), recipe.clone());
        let serialized_recipe = serde_json::to_string(&recipe).unwrap();
        self.vault.collect("recipe", &recipe.name, &serialized_recipe);
    }

    /// Retrieves a recipe by its name.
    pub fn get_recipe(&self, name: &str) -> Option<Recipe> {
        if let Some(recipe) = self.recipes.get(name) {
            Some(recipe.clone())
        } else {
            if let Some(data) = self.vault.skim("recipe", name) {
                let recipe: Recipe = serde_json::from_str(&data).unwrap();
                Some(recipe)
            } else {
                None
            }
        }
    }

    pub fn get_recipes_for_crafter(&self, crafter: &Crafter) -> Vec<&Recipe> {
        self.crafters.get(crafter)
            .map(|recipe_names| recipe_names.iter().filter_map(|name| self.get_recipe(name)).collect())
            .unwrap_or_else(Vec::new)
    }

    /// Checks if a recipe can be crafted with the given inventory.
    pub fn can_craft(&self, recipe_name: &str, inventory: &HashMap<String, Ingredient>) -> bool {
        if let Some(recipe) = self.get_recipe(recipe_name) {
            for ingredient in &recipe.ingredients {
                if let Some(inventory_ingredient) = inventory.get(&ingredient.name) {
                    if !inventory_ingredient.recipe_craftable || inventory_ingredient.quantity < ingredient.quantity {
                        return false;    // Not enough of this ingredient in inventory or not craftable
                    }
                } else {
                    return false;    // Missing this ingredient in inventory
                }
            }
            true    // Can craft the recipe
        } else {
            false    // Recipe not found in the RecipeBook
        }
    }

    /// Crafts a recipe if possible, updating the inventory.
    pub async fn craft(&mut self, recipe_name: &str, inventory: &mut HashMap<String, Ingredient>) -> Option<String> {
        if self.can_craft(recipe_name, inventory) {
            let recipe = self.get_recipe(recipe_name).unwrap().clone();    // Clone the recipe
            for ingredient in &recipe.ingredients {
                if let Some(inventory_ingredient) = inventory.get_mut(&ingredient.name) {
                    if inventory_ingredient.recipe_craftable {
                        inventory_ingredient.quantity -= ingredient.quantity;
                    }
                }
            }
            // Simulate crafting time
            sleep(Duration::from_secs(recipe.base_cook_time.into())).await;

            // Increment cook count and check if recipe is mastered
            let mut recipe = self.recipes.get_mut(recipe_name).unwrap();
            recipe.increment_cook_count();

            // Send cook count update to PebbleVault
            let serialized_recipe = serde_json::to_string(&recipe).unwrap();
            self.vault.collect("recipe", &recipe.name, &serialized_recipe);

            // Notify SkillsScript if the recipe is mastered
            if recipe.is_mastered() {
                self.vault.collect("mastered_recipe", &recipe.name, r#"{"mastered": true}"#);
            }

            // Notify SkillsScript about skill experience gain
            self.vault.collect("skill_experience", &recipe.name, r#"{"experience": 10}"#);

            Some(recipe.outcome.clone())    // Return the outcome of crafting
        } else {
            None    // Return None if crafting fails
        }
    }

    /// Imports recipes from a JSON or CSV file.
    pub fn import_recipes_from_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        if filename ends_with(".json") {
            // Parse JSON
            let recipes: Vec<Recipe> = serde_json::from_reader(reader)?;
            for recipe in recipes {
                self.add_recipe(recipe);
            }
        } else if filename ends_with(".csv") {
            // Parse CSV
            let mut csv_reader = Reader::from_reader(reader);
            for result in csv_reader.deserialize::<Recipe>() {
                let recipe = result?;
                self.add_recipe(recipe);
            }
        } else {
            return Err(Box::new(Error::new(ErrorKind::Other, "Unsupported file format")));
        }

        Ok(())
    }
}

#[tokio::main]
pub async fn main() {
    // Initialize logger
    env_logger::init();

    // Setup error log file
    let log_dir = "errorlogs";
    let log_file_path = format!("{}/lastlog.txt", log_dir);

    // Ensure log directory exists
    fs::create_dir_all(log_dir).unwrap();

    // Open log file
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
        .unwrap();

    let mut recipe_book = RecipeBook::new();


    
    // Load recipes from files
    if let Err(e) = recipe_book.import_recipes_from_file("recipes.json") {
        error!("Error importing recipes from JSON: {}", e);
    }

    if let Err(e) = recipe_book.import_recipes_from_file("recipes.csv") {
        error!("Error importing recipes from CSV: {}", e);
    }

    let inventory: Arc<Mutex<HashMap<String, Ingredient>>> = Arc::new(Mutex::new(HashMap::new()));
    let recipe_book = Arc::new(Mutex::new(recipe_book));



    // Initialization of inventory ingredients found in json with quantities and craftable status
    let items = [
        ("Herb", 3, true),
        ("Water", 10, true),
        ("Flour", 10, true),
        ("Salt", 5, true),
        ("Sugar", 5, true),
        ("Egg", 5, true),
        ("Milk", 5, true),
        ("Meat", 5, true),
        ("Potato", 5, true),
        ("Carrot", 5, true),
        ("Lettuce", 5, true),
        ("Tomato", 5, true),
        ("Cucumber", 5, true),
        ("Olive Oil", 5, true),
        ("Ham", 5, true),
        ("Cheese", 5, true),
        ("Yeast", 5, true),
        ("Butter", 5, true),
        ("Beef", 5, true),
        ("Onion", 5, true),
        ("Pepper", 5, true),
        ("Celery", 5, true),
        ("Chicken", 5, true),
        ("Garlic", 5, true),
        ("Rosemary", 5, true),
        ("Lemon", 5, true),
        ("Fish", 5, true),
        ("Apple", 5, true),
        ("Banana", 5, true),
        ("Orange", 5, true),
        ("Honey", 5, true),
        ("Berry", 5, true),
        ("Oil", 5, true),
        ("Rice", 5, true),
        ("Peas", 5, true),
        ("Soy Sauce", 5, true),
        ("Curry Powder", 5, true),
        ("Coconut Milk", 5, true),
        ("Ginger", 5, true),
        ("Pasta", 5, true),
        ("Ground Beef", 5, true),
        ("Vinegar", 5, true),
        ("Sausage", 5, true),
        ("Dough", 5, true),
        ("Tomato Sauce", 5, true),
        ("Pepperoni", 5, true),
        ("Cinnamon", 5, true),
        ("Mushroom", 5, true),
        ("Tortilla", 5, true),
        ("Mayonnaise", 5, true),
    ];

    {
        let mut inventory = inventory.lock().unwrap();
        for (name, quantity, recipe_craftable) in &items {
            inventory.insert(
                name.to_string(),
                Ingredient {
                    name: name.to_string(),
                    quantity: *quantity,
                    recipe_craftable: *recipe_craftable,
                },
            );
        }
    }


    // Example crafters
    let furnace = Crafter { name: "Furnace".to_string() };
    let workbench = Crafter { name: "Workbench".to_string() };


    // Adding recipes with crafters
    {
        let mut recipe_book = recipe_book.lock().unwrap();
        recipe_book.add_recipe(Recipe {
            name: "Bread".to_string(),
            ingredients: vec![
                Ingredient {
                    name: "Flour".to_string(),
                    quantity: 2,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Water".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Salt".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Yeast".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
            ],
            outcome: "Bread".to_string(),
            crafters: vec![furnace.clone(), workbench.clone()],
            base_cook_time: 30,
            cook_count: 0,
        });

        recipe_book.add_recipe(Recipe {
            name: "Cake".to_string(),
            ingredients: vec![
                Ingredient {
                    name: "Flour".to_string(),
                    quantity: 3,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Sugar".to_string(),
                    quantity: 2,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Egg".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Milk".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Butter".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
            ],
            outcome: "Cake".to_string(),
            crafters: vec![furnace.clone(), workbench.clone()],
            base_cook_time: 45,
            cook_count: 0,
        });

        recipe_book.add_recipe(Recipe {
            name: "Health Potion".to_string(),
            ingredients: vec![
                Ingredient {
                    name: "Herb".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
                Ingredient {
                    name: "Water".to_string(),
                    quantity: 1,
                    recipe_craftable: true,
                },
            ],
            outcome: "Health Potion".to_string(),
            crafters: vec![Crafter { name: "Alchemy Table".to_string() }],
            base_cook_time: 5,
            cook_count: 0,
        });

        // Additional recipes can be added similarly...
    }


    // Retrieve recipes for a specific crafter
    {
        let recipe_book = recipe_book.lock().unwrap();
        let furnace_recipes = recipe_book.get_recipes_for_crafter(&furnace);
        println!("Recipes for Furnace:");
        for recipe in furnace_recipes {
            println!("{:?}", recipe);
        }
    }

    loop {
        // Simulate crafting attempts
        {
            let mut inventory = inventory.lock().unwrap();
            let mut recipe_book = recipe_book.lock().unwrap();

            // Example crafting process
            if recipe_book.can_craft("Bread", &inventory) {
                if let Some(item) = recipe_book.craft("Bread", &mut inventory).await {
                    info!("Crafted: {}", item);
                    recipe_book.vault.collect("skill_experience", "crafting", r#"{"experience": 10}"#);
                } else {
                    error!("Failed to craft Bread.");
                }
            } else {
                error!("Not enough ingredients to craft Bread.");
            }

            if recipe_book.can_craft("Cake", &inventory) {
                if let Some(item) = recipe_book.craft("Cake", &mut inventory).await {
                    info!("Crafted: {}", item);
                    recipe_book.vault.collect("skill_experience", "crafting", r#"{"experience": 20}"#);
                } else {
                    error!("Failed to craft Cake.");
                }
            } else {
                error!("Not enough ingredients to craft Cake.");
            }
        }

        // Sleep to simulate ongoing process
        sleep(Duration::from_secs(5)).await;
    }
}