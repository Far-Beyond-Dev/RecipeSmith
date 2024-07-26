use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use csv::Reader;
use serde_json::Value;
use uuid::Uuid;

/// Placeholder function to simulate object creation in a memory database.
fn create_object() -> Uuid {
    // Simulate creating an object and returning its UUID
    Uuid::new_v4()
}

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
    pub crafters: Vec<String>,  // Updated to Vec<String> for crafters
    pub recipe_craftable: bool,  // Added field for recipe_craftable
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
            uuid: create_object(),
            inventory: PlayerInventory::new(num_slots),
        }
    }
}

/// Struct representing a Recipe Book.
pub struct RecipeBook {
    pub recipes: HashMap<String, Recipe>,
    pub crafters: HashMap<Crafter, Vec<String>>,
}

////////////////////////////////////////////
//               Recipe Book              //
//  This is where recipes are added,      //
//  and indexed                           //
////////////////////////////////////////////

impl RecipeBook {
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
            crafters: HashMap::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        for crafter_name in &recipe.crafters {
            let crafter = Crafter { name: crafter_name.clone() }; // Convert string to Crafter
            self.crafters.entry(crafter).or_insert_with(Vec::new).push(recipe.name.clone());
        }
        self.recipes.insert(recipe.name.clone(), recipe);
    }

    pub fn get_recipe(&self, name: &str) -> Option<&Recipe> {
        self.recipes.get(name)
    }

    pub fn get_recipes_for_crafter(&self, crafter: &Crafter) -> Vec<&Recipe> {
        self.crafters.get(crafter)
            .map(|recipe_names| recipe_names.iter().filter_map(|name| self.get_recipe(name)).collect())
            .unwrap_or_else(Vec::new)
    }

    pub fn can_craft(&self, recipe_name: &str, inventory: &HashMap<String, Ingredient>) -> bool {
        if let Some(recipe) = self.get_recipe(recipe_name) {
            for ingredient in &recipe.ingredients {
                if let Some(inventory_ingredient) = inventory.get(&ingredient.name) {
                    if !inventory_ingredient.recipe_craftable || inventory_ingredient.quantity < ingredient.quantity {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn craft(&self, recipe_name: &str, inventory: &mut HashMap<String, Ingredient>) -> Option<String> {
        if self.can_craft(recipe_name, inventory) {
            let recipe = self.get_recipe(recipe_name).unwrap().clone();
            for ingredient in &recipe.ingredients {
                if let Some(inventory_ingredient) = inventory.get_mut(&ingredient.name) {
                    if inventory_ingredient.recipe_craftable {
                        inventory_ingredient.quantity -= ingredient.quantity;
                    }
                }
            }
            Some(recipe.outcome.clone())
        } else {
            None
        }
    }

    pub fn import_recipes_from_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        if filename.ends_with(".json") {
            let json_data: Value = serde_json::from_reader(reader)?;
            if let Some(recipes) = json_data.get("recipes").and_then(Value::as_array) {
                for recipe in recipes {
                    let recipe: Recipe = serde_json::from_value(recipe.clone())?;
                    self.add_recipe(recipe);
                }
            } else {
                return Err(Box::new(Error::new(ErrorKind::InvalidData, "Invalid JSON format")));
            }
        } else if filename.ends_with(".csv") {
            // Handle CSV import (assuming the same structure is present in CSV)
            // Note: You may need to adjust this based on the CSV structure.
            return Err(Box::new(Error::new(ErrorKind::Other, "CSV import not implemented")));
        } else {
            return Err(Box::new(Error::new(ErrorKind::Other, "Unsupported file format")));
        }

        Ok(())
    }
}


/////////////////////////////////////////////////////////
//                TESTING PURPOSES ONLY!!!             //
//  This is to test that the system works correctly    //
/////////////////////////////////////////////////////////

fn main() {
    // Example usage:
    let mut recipe_book = RecipeBook::new();

    // Import recipes from JSON
    if let Err(e) = recipe_book.import_recipes_from_file("recipes/recipes.json") {
        eprintln!("Error importing recipes: {}", e);
    }

    // Import recipes from CSV
    if let Err(e) = recipe_book.import_recipes_from_file("recipes/recipes.csv") {
        eprintln!("Error importing recipes: {}", e);
    }

    // Initialize inventory with ingredients and their quantities
    let mut inventory: HashMap<String, Ingredient> = HashMap::new();

    // Example initialization of inventory ingredients with quantities and craftable status
    let items = [
        ("Herb", 3, true),
        ("Water", 2, true),
        ("Flour", 4, true),
        ("Salt", 2, true),
        ("Sugar", 3, true),
        ("Egg", 4, true),
        ("Milk", 1, true),
        ("Meat", 2, true),
        ("Potato", 3, true),
        ("Carrot", 2, true),
        ("Lettuce", 3, true),
        ("Tomato", 2, true),
        ("Cucumber", 1, true),
        ("Olive Oil", 1, true),
        ("Ham", 1, true),
        ("Cheese", 1, true),
    ];

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

    // Example crafters
    let furnace = Crafter { name: "Furnace".to_string() };
    let workbench = Crafter { name: "Workbench".to_string() };

    // Adding recipes with crafters as strings
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
        ],
        outcome: "Bread".to_string(),
        crafters: vec!["Oven".to_string(), "Bakery".to_string()],
        recipe_craftable: true,
    });

    // Retrieve recipes for a specific crafter
    let furnace_recipes = recipe_book.get_recipes_for_crafter(&furnace);
    println!("Recipes for Furnace:");
    for recipe in furnace_recipes {
        println!("{:?}", recipe);
    }

    // Attempt to craft Bread
    if recipe_book.can_craft("Bread", &inventory) {
        if let Some(item) = recipe_book.craft("Bread", &mut inventory) {
            println!("Crafted: {}", item);
        } else {
            println!("Failed to craft Bread.");
        }
    } else {
        println!("Not enough ingredients to craft Bread.");
    }
}
