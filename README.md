# RecipeSmith

RecipeSmith is a Rust-based module designed for managing recipes, ingredients, and crafting systems in games. It provides a flexible framework for creating, storing, retrieving, and crafting recipes based on available ingredients.

## Features

- **Dynamic Recipe Management:** Create, store, retrieve, and craft recipes.
- **Ingredient Interaction:** Detailed tracking and management of ingredients, including quantities and craftability.
- **Crafter System:** Associate recipes with specific crafters or crafting stations.
- **Inventory Management:** Built-in player inventory and storage container systems.
- **Recipe Book:** Centralized storage and indexing of recipes.
- **File Import:** Import recipes from JSON files.
- **Flexible Data Structures:** Support for various game objects like items, crafters, and storage containers.

## Core Structures

### Ingredient

```rust
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub recipe_craftable: bool,
}
```

Represents an ingredient with its name, required quantity, and whether it's craftable through recipes.

### Recipe

```rust
pub struct Recipe {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub outcome: String,
    pub crafters: Vec<String>,
    pub recipe_craftable: bool,
}
```

Defines a recipe with its name, required ingredients, outcome, associated crafters, and craftability status.

### RecipeBook

```rust
pub struct RecipeBook {
    pub recipes: HashMap<String, Recipe>,
    pub crafters: HashMap<Crafter, Vec<String>>,
}
```

Central structure for managing recipes and their associations with crafters.

## Key Functionalities

### Adding Recipes

```rust
impl RecipeBook {
    pub fn add_recipe(&mut self, recipe: Recipe) {
        // Implementation details...
    }
}
```

Add new recipes to the RecipeBook, automatically indexing them by crafter.

### Crafting

```rust
impl RecipeBook {
    pub fn can_craft(&self, recipe_name: &str, inventory: &HashMap<String, Ingredient>) -> bool {
        // Implementation details...
    }

    pub fn craft(&self, recipe_name: &str, inventory: &mut HashMap<String, Ingredient>) -> Option<String> {
        // Implementation details...
    }
}
```

Check if a recipe can be crafted and perform the crafting operation, updating the inventory.

### Importing Recipes

```rust
impl RecipeBook {
    pub fn import_recipes_from_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation details...
    }
}
```

Import recipes from JSON files, allowing for easy expansion of the recipe database.

## Inventory Management

### PlayerInventory

```rust
pub struct PlayerInventory {
    pub slots: HashMap<u32, Option<Item>>,
}
```

Manages player inventories with methods for adding, removing, and retrieving items.

### StorageContainer

```rust
pub struct StorageContainer {
    pub uuid: Uuid,
    pub inventory: PlayerInventory,
}
```

Represents storage containers in the game world, each with its own inventory.

## Usage Example

```rust
fn main() {
    let mut recipe_book = RecipeBook::new();

    // Import recipes
    recipe_book.import_recipes_from_file("recipes/recipes.json").unwrap_or_else(|e| {
        eprintln!("Error importing recipes: {}", e);
    });

    // Initialize inventory
    let mut inventory: HashMap<String, Ingredient> = HashMap::new();
    // Add ingredients to inventory...

    // Attempt to craft
    if recipe_book.can_craft("Bread", &inventory) {
        if let Some(item) = recipe_book.craft("Bread", &mut inventory) {
            println!("Crafted: {}", item);
        }
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
