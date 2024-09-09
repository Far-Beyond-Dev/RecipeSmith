# RecipeSmith

RecipeSmith is a rs-based module designed by: Asterisk for use with games that utilize the stars beyond horizon Server Backend as a framework for managing recipes, ingredients, and their interactions within a recipe book. This system integrates with PebbleVault for data storage and retrieval while being capable of communication with other submodules, ensuring modularity and independence. The goal is to enable dynamic recipe management, including creating, storing, retrieving, and crafting recipes based on available ingredients.

## Features

- **Dynamic Recipe Management:** Create, store, retrieve, and craft recipes based on available ingredients.
- **Integration with PebbleVault:** Seamless data storage and retrieval with PebbleVault.
- **Modularity:** Designed to communicate with other submodules independently.
- **Ingredient Interaction:** Detailed tracking and management of ingredients and their quantities.
- **Outcome Prediction:** Automatically determine the outcome of recipes based on input ingredients.
- **Loosely Coupled Architecture:** Ensures components can function independently, enhancing flexibility and scalability.
- **Full Error Logging Functionality:** Comprehensive error logging to facilitate debugging and maintain system integrity.
- **Extendable and Customizable:** Easily extendable to add new features or customize existing ones to fit specific needs.

## How It Works

### 1. Dynamic Recipe Management

#### Definition

Recipes are defined in a JSON file containing the name, ingredients, and outcome of each recipe. Here is an example structure:

```json
{
    "recipes": [
        {
            "name": "Bread",
            "ingredients": [
                {"name": "Flour", "quantity": 2},
                {"name": "Water", "quantity": 1},
                {"name": "Salt", "quantity": 1},
                {"name": "Yeast", "quantity": 1}
            ],
            "outcome": "Bread",
            "crafters": ["Oven", "Bakery"],
            "recipe_craftable": true,
            "base_cook_time": 30,
            "cook_count": 0
        }
        // Add other recipes
    ]
}
```

#### Recipe Structure

The `Recipe` struct represents a recipe with its name, ingredients, outcome, crafters, base cook time, and cook count.

```rs
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

Recipes can be added to the `RecipeBook` using the `add_recipe` method.

```rs
pub fn add_recipe(&mut self, recipe: Recipe) {
    for crafter in &recipe.crafters {
        self.crafters.entry(crafter.clone()).or_insert_with(Vec::new).push(recipe.name.clone());
    }
    self.recipes.insert(recipe.name.clone(), recipe.clone());
    let serialized_recipe = serde_json::to_string(&recipe).unwrap();
    self.vault.collect("recipe", &recipe.name, &serialized_recipe);
}
```

### 2. Integration with PebbleVault

#### Seamless Data Storage and Retrieval

Recipes and ingredients are securely stored and easily accessible through integration with PebbleVault. This ensures efficient operation within a larger ecosystem, enabling modular and independent submodule communication.

```rs
let vault = Vault::new();
vault.define_class("recipe", r#"{
    "name": "string",
    "ingredients": "array",
    "outcome": "string",
    "base_cook_time": "u32",
    "cook_count": "u32",
    "crafters": "array"
}"#);
```

### 3. Modularity

#### Independent Communication

RecipeSmith is designed to communicate with other submodules independently, allowing for flexible and scalable integration into various projects.

### 4. Ingredient Interaction

#### Detailed Tracking and Management

The `Ingredient` struct represents an ingredient with its name, quantity, and whether it can be crafted from a recipe.

```rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub recipe_craftable: bool,
}
```

### 5. Outcome Prediction

#### Automatically Determine Recipe Outcomes

When crafting a recipe, the outcome is determined based on the input ingredients.

```rs
pub fn craft(&mut self, recipe_name: &str, inventory: &mut HashMap<String, Ingredient>) -> Option<String> {
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
```

### 6. Loosely Coupled Architecture

#### Flexibility and Scalability

Components can function independently, which enhances the flexibility and scalability of the system.

### 7. Full Error Logging Functionality

#### Comprehensive Error Logging

Error logging captures and logs errors to facilitate debugging and maintain system integrity. This feature ensures that any issues can be quickly identified and resolved.

```rs
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
```

### 8. Extendable and Customizable

#### Adding New Features

RecipeSmith is easily extendable to add new features or customize existing ones to fit specific needs. This flexibility makes it suitable for a wide range of applications.

## Example Usage

### Initialization

1. **Clone the Repository:**
   ```sh
   git clone https://github.com/Stars-Beyond/RecipeSmith.git
   ```

2. **Navigate to the Project Directory:**
   ```sh
   cd RecipeSmith
   ```

3. **Build the Project:**
   ```sh
   cargo build
   ```

4. **Run the Project:**
   ```sh
   cargo run
   ```

### Recipe Management

#### Adding a Recipe

```rs
let mut recipe_book = RecipeBook::new();
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
    crafters: vec![Crafter { name: "Oven".to_string() }],
    base_cook_time: 30,
    cook_count: 0,
});
```

#### Crafting a Recipe

```rs
let mut inventory = HashMap::new();
inventory.insert("Flour".to_string(), Ingredient {
    name: "Flour".to_string(),
    quantity: 2,
    recipe_craftable: true,
});
inventory.insert("Water".to_string(), Ingredient {
    name: "Water".to_string(),
    quantity: 1,
    recipe_craftable: true,
});

if recipe_book.can_craft("Bread", &inventory) {
    if let Some(outcome) = recipe_book.craft("Bread", &mut inventory) {
        println!("Crafted: {}", outcome);
    } else {
        println!("Failed to craft Bread.");
    }
} else {
    println!("Not enough ingredients to craft Bread.");
}
```

## SkillScript Integration

### Experience Gain and Reducing Cooking Time

SkillScript can be integrated to give players experience over time, reduce cooking time, or gain rewards. This can be done by collecting experience points and updating the crafting system.

```rs
if recipe_book.can_craft("Bread", &inventory) {
    if let Some(outcome) = recipe_book.craft("Bread", &mut inventory) {
        recipe_book.vault.collect("skill_experience", "crafting", r#"{"experience": 10}"#);
        println!("Crafted: {}", outcome);
    } else {
        println!("Failed to craft Bread.");
    }
} else {
    println!("Not enough ingredients to craft Bread.");
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.