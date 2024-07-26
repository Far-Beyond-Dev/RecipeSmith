# RecipeSmith

RecipeSmith is a Rust-based module designed by asterisk for use with games that utilize the stars beyond horizon Server Backend as a framework for managing recipes, ingredients, and their interactions within a recipe book. This system integrates with PebbleVault for data storage and retrieval while being capable of communication with other submodules, ensuring modularity and independence. The goal is to enable dynamic recipe management, including creating, storing, retrieving, and crafting recipes based on available ingredients.

## Features

- **Dynamic Recipe Management:** Create, store, retrieve, and craft recipes based on available ingredients.
- **Integration with PebbleVault:** Seamless data storage and retrieval with PebbleVault.
- **Modularity:** Designed to communicate with other submodules independently.
- **Ingredient Interaction:** Detailed tracking and management of ingredients and their quantities.
- **Outcome Prediction:** Automatically determine the outcome of recipes based on input ingredients.
- **Loosely Coupled Architecture:** Ensures components can function independently, enhancing flexibility and scalability.
- **Full Error Logging Functionality:** Comprehensive error logging to facilitate debugging and maintain system integrity.
- **Server Independence:** Operates without dependency on a central server, allowing for distributed and offline use cases.
- **Extendable and Customizable:** Easily extendable to add new features or customize existing ones to fit specific needs.

## How It Works

### Recipe Structure

Recipes are defined in a JSON file containing the name, ingredients, and outcome of each recipe. Here is an example structure:

```json
{
    "_comment": "Recipes JSON file containing various recipes and their ingredients.",
    "recipes": [
        {
            "name": "Bread",
            "ingredients": [
                {"name": "Flour", "quantity": 2},
                {"name": "Water", "quantity": 1},
                {"name": "Salt", "quantity": 1}
            ],
            "outcome": "Bread"
        },
        {
            "name": "Cake",
            "ingredients": [
                {"name": "Flour", "quantity": 3},
                {"name": "Sugar", "quantity": 2},
                {"name": "Egg", "quantity": 1},
                {"name": "Milk", "quantity": 1}
            ],
            "outcome": "Cake"
        },
        {
            "name": "Potion",
            "ingredients": [
                {"name": "Herb", "quantity": 1},
                {"name": "Water", "quantity": 1}
            ],
            "outcome": "Health Potion"
        },
        {
            "name": "Stew",
            "ingredients": [
                {"name": "Meat", "quantity": 2},
                {"name": "Potato", "quantity": 3},
                {"name": "Carrot", "quantity": 2},
                {"name": "Water", "quantity": 2}
            ],
            "outcome": "Stew"
        },
        {
            "name": "Salad",
            "ingredients": [
                {"name": "Lettuce", "quantity": 2},
                {"name": "Tomato", "quantity": 2},
                {"name": "Cucumber", "quantity": 1},
                {"name": "Olive Oil", "quantity": 1}
            ],
            "outcome": "Salad"
        },
        {
            "name": "Sandwich",
            "ingredients": [
                {"name": "Bread", "quantity": 2},
                {"name": "Ham", "quantity": 1},
                {"name": "Cheese", "quantity": 1},
                {"name": "Lettuce", "quantity": 1}
            ],
            "outcome": "Sandwich"
        }
    ]
}
```

### Integration with PebbleVault

RecipeSmith integrates with PebbleVault for data storage and retrieval, ensuring that all recipes and ingredients are securely stored and easily accessible. This integration allows RecipeSmith to operate efficiently within a larger ecosystem, enabling modular and independent submodule communication.

### Modularity and Independence

RecipeSmith is designed to be modular and independent, capable of interacting with other submodules within the system. This design ensures that RecipeSmith can be easily integrated into various projects, providing a flexible and scalable solution for recipe management.

### Error Logging

RecipeSmith includes a comprehensive error logging functionality that captures and logs errors to facilitate debugging and maintain system integrity. This feature ensures that any issues can be quickly identified and resolved.

### Server Independence

RecipeSmith operates without dependency on a central server, allowing for distributed and offline use cases. This feature makes it ideal for applications that require high availability and resilience.

### Extendability and Customization

RecipeSmith is easily extendable and customizable, allowing developers to add new features or modify existing ones to fit specific needs. This flexibility makes it suitable for a wide range of applications.

## Getting Started
                                       
1. **Clone the Repository:**
   ```sh
   git clone https://github.com/Stars-Beyond/RecipeSmith
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

## Example Usage in an RPG Game

### Integration into an RPG Game

1. **Define Recipes and Ingredients:**
   - Use the JSON structure provided by RecipeSmith to define various recipes and their required ingredients.

2. **Crafting System:**
   - Implement a crafting system in your RPG game that interacts with RecipeSmith. Players can collect ingredients and use the crafting system to create new items.

3. **Storage and Retrieval:**
   - Integrate PebbleVault to store and retrieve recipes and ingredients, allowing players to save their progress and access it later.

4. **Outcome Determination:**
   - Use RecipeSmith’s outcome prediction feature to automatically determine the result of combining different ingredients.

### Example Workflow

1. **Collect Ingredients:**
   - Players gather ingredients during gameplay (e.g., Flour, Water, Salt).

2. **Open Crafting Menu:**
   - Players open the crafting menu where they can see available recipes.

3. **Select Recipe:**
   - Players select a recipe (e.g., Bread) and see the required ingredients.

4. **Craft Item:**
   - Players combine the required ingredients and craft the item, resulting in the outcome (e.g., Bread).

### Example Code Snippet

```rs
use recipesmith::{Recipe, Ingredient};

fn main() {
    let flour = Ingredient::new("Flour", 2);
    let water = Ingredient::new("Water", 1);
    let salt = Ingredient::new("Salt", 1);

    let bread_recipe = Recipe::new("Bread", vec![flour, water, salt], "Bread");

    let crafted_item = bread_recipe.craft();
    println!("Crafted Item: {}", crafted_item);
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.


RecipeSmith is a powerful tool for managing and interacting with recipes and ingredients. With its robust features and seamless integration with PebbleVault, it provides a dynamic and flexible solution for any recipe management needs you may have.
