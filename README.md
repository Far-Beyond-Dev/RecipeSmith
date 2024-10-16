# RecipeSmith

RecipeSmith is a Rust-based plugin designed for use with games that utilize the Stars Beyond Horizon Server Backend. It provides a framework for managing recipes, ingredients, and their interactions within a recipe book. This system is designed to be modular and independent, capable of communicating with other plugins through custom events. The goal is to enable dynamic recipe management, including creating, storing, retrieving, and crafting recipes based on available ingredients.

## Features

- **Dynamic Recipe Management:** Create, store, retrieve, and craft recipes based on available ingredients.
- **Event-Driven Architecture:** Utilizes custom events for communication with other plugins.
- **Ingredient Interaction:** Detailed tracking and management of ingredients and their quantities.
- **Outcome Prediction:** Automatically determine the outcome of recipes based on input ingredients.
- **Loosely Coupled Architecture:** Ensures components can function independently, enhancing flexibility and scalability.
- **Full Error Logging Functionality:** Comprehensive error logging to facilitate debugging and maintain system integrity.
- **Extendable and Customizable:** Easily extendable to add new features or customize existing ones to fit specific needs.

## How It Works

### 1. Dynamic Recipe Management

#### Recipe Structure

The `Recipe` struct represents a recipe with its name, ingredients, outcome, crafters, base cook time, and cook count.

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipe {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub outcome: String,
    pub crafters: Vec<Crafter>,
    pub base_cook_time: u32,
    pub cook_count: u32,
}
```

#### Adding Recipes

Recipes can be added to the `RecipeBook` using the `add_new_recipe` method.

```rust
pub async fn add_new_recipe(&self, recipe: Recipe) {
    let mut recipe_book = self.recipe_book.write().await;
    recipe_book.add_recipe(recipe);
}
```

### 2. Event-Driven Architecture

RecipeSmith uses custom events for communication. Here are some of the key events:

- `craft_item`: Attempts to craft an item for a player.
- `add_recipe`: Adds a new recipe to the recipe book.
- `get_player_inventory`: Retrieves a player's inventory contents.
- `add_item_to_inventory`: Adds an item to a player's inventory.
- `remove_item_from_inventory`: Removes an item from a player's inventory.
- `create_storage_container`: Creates a new storage container.

### 3. Ingredient Interaction

The `Ingredient` struct represents an ingredient with its name, quantity, and whether it can be crafted from a recipe.

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub recipe_craftable: bool,
}
```

### 4. Outcome Prediction

When crafting a recipe, the outcome is determined based on the input ingredients.

```rust
pub async fn craft_item(&self, player_id: &str, recipe_name: &str, context: &mut PluginContext) -> Option<String> {
    // Implementation details...
}
```

## Example Usage

### Initialization

The RecipeSmith plugin is initialized as part of the plugin system. It automatically registers for custom events and loads recipes from files.

### Recipe Management

#### Adding a Recipe

To add a new recipe, emit a custom event:

```rust
let new_recipe = Recipe {
    name: "Bread".to_string(),
    ingredients: vec![
        Ingredient { name: "Flour".to_string(), quantity: 2, recipe_craftable: true },
        Ingredient { name: "Water".to_string(), quantity: 1, recipe_craftable: true },
    ],
    outcome: "Bread".to_string(),
    crafters: vec![Crafter { name: "Oven".to_string() }],
    base_cook_time: 30,
    cook_count: 0,
};

context.dispatch_custom_event(CustomEvent {
    event_type: "add_recipe".to_string(),
    data: Arc::new(new_recipe),
}).await;
```

#### Crafting a Recipe

To craft a recipe, emit a custom event:

```rust
context.dispatch_custom_event(CustomEvent {
    event_type: "craft_item".to_string(),
    data: Arc::new(("player1".to_string(), "Bread".to_string())),
}).await;
```

### Inventory Management

#### Adding an Item to Inventory

```rust
let new_item = Item { /* ... */ };
context.dispatch_custom_event(CustomEvent {
    event_type: "add_item_to_inventory".to_string(),
    data: Arc::new(("player1".to_string(), new_item)),
}).await;
```

#### Removing an Item from Inventory

```rust
context.dispatch_custom_event(CustomEvent {
    event_type: "remove_item_from_inventory".to_string(),
    data: Arc::new(("player1".to_string(), "Bread".to_string())),
}).await;
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
