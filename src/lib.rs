use plugin_test_api::{PluginInformation, SayHello, BaseAPI, GameEvent, CustomEvent, PluginContext, Plugin};
use std::{net::ToSocketAddrs, sync::Arc};
use async_trait::async_trait;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use horizon_data_types::Player;
use ez_logging::println;
use csv;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub recipe_craftable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Crafter {
    pub name: String,
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub model: Option<String>,
    pub meta_tags: HashMap<String, serde_json::Value>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeBook {
    pub recipes: HashMap<String, Recipe>,
    pub crafters: HashMap<Crafter, Vec<String>>,
}

impl Clone for RecipeSmith {
    fn clone(&self) -> Self {
        RecipeSmith {
            initialized: self.initialized,
            recipe_book: Arc::clone(&self.recipe_book),
            player_inventories: Arc::clone(&self.player_inventories),
        }
    }
}

impl RecipeBook {
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
            crafters: HashMap::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        for crafter in &recipe.crafters {
            self.crafters.entry(crafter.clone()).or_insert_with(Vec::new).push(recipe.name.clone());
        }
        self.recipes.insert(recipe.name.clone(), recipe);
    }

    pub fn get_recipe(&self, name: &str) -> Option<Recipe> {
        self.recipes.get(name).cloned()
    }

    pub fn get_recipes_for_crafter(&self, crafter: &Crafter) -> Vec<Recipe> {
        self.crafters.get(crafter)
            .map(|recipe_names| recipe_names.iter().filter_map(|name| self.get_recipe(name)).collect())
            .unwrap_or_else(Vec::new)
    }

    pub fn can_craft(&self, recipe_name: &str, inventory: &HashMap<String, Ingredient>) -> bool {
        if let Some(recipe) = self.get_recipe(recipe_name) {
            recipe.ingredients.iter().all(|ingredient| {
                inventory.get(&ingredient.name)
                    .map(|inv_ingredient| inv_ingredient.recipe_craftable && inv_ingredient.quantity >= ingredient.quantity)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    pub async fn craft(&mut self, recipe_name: &str, inventory: &mut HashMap<String, Ingredient>) -> Option<String> {
        if self.can_craft(recipe_name, inventory) {
            let recipe = self.get_recipe(recipe_name)?;
            
            // Consume ingredients
            for ingredient in &recipe.ingredients {
                if let Some(inv_ingredient) = inventory.get_mut(&ingredient.name) {
                    inv_ingredient.quantity -= ingredient.quantity;
                }
            }

            // Simulate crafting time
            tokio::time::sleep(tokio::time::Duration::from_secs(recipe.base_cook_time.into())).await;

            // Update recipe
            if let Some(recipe) = self.recipes.get_mut(recipe_name) {
                recipe.increment_cook_count();
            }

            Some(recipe.outcome.clone())
        } else {
            None
        }
    }

    pub fn import_recipes_from_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);

        if filename.ends_with(".json") {
            let recipes: Vec<Recipe> = serde_json::from_reader(reader)?;
            for recipe in recipes {
                self.add_recipe(recipe);
            }
        } else if filename.ends_with(".csv") {
            let mut csv_reader = csv::Reader::from_reader(reader);
            for result in csv_reader.deserialize() {
                let recipe: Recipe = result?;
                self.add_recipe(recipe);
            }
        } else {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported file format")));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RecipeSmith {
    initialized: bool,
    recipe_book: Arc<RwLock<RecipeBook>>,
    player_inventories: Arc<RwLock<HashMap<String, PlayerInventory>>>,
}

impl RecipeSmith {
    pub fn new() -> Self {
        Self {
            initialized: false,
            recipe_book: Arc::new(RwLock::new(RecipeBook::new())),
            player_inventories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn initialize_recipe_smith(&mut self, context: &mut PluginContext) {
        if !self.initialized {
            println!("RecipeSmith initializing...");
            self.register_custom_event("recipe_learned", context).await;
            self.register_custom_event("item_crafted", context).await;
            self.register_custom_event("inventory_changed", context).await;
            self.register_custom_event("recipe_mastered", context).await;
            self.register_custom_event("crafting_failed", context).await;
            self.register_custom_event("storage_container_created", context).await;
            self.register_custom_event("storage_container_accessed", context).await;

            // Load recipes from files
            let mut recipe_book = self.recipe_book.write().await;
            if let Err(e) = recipe_book.import_recipes_from_file("recipes.json") {
                println!("Error importing recipes from JSON: {}", e);
            }
            if let Err(e) = recipe_book.import_recipes_from_file("recipes.csv") {
                println!("Error importing recipes from CSV: {}", e);
            }

            self.initialized = true;
            println!("RecipeSmith initialized!");
        }
    }

    async fn create_player_inventory(&self, player_id: &str, num_slots: u32) {
        let mut inventories = self.player_inventories.write().await;
        inventories.insert(player_id.to_string(), PlayerInventory::new(num_slots));
    }

    async fn get_player_inventory(&self, player_id: &str) -> Option<PlayerInventory> {
        let inventories = self.player_inventories.read().await;
        inventories.get(player_id).cloned()
    }

    async fn update_player_inventory(&self, player_id: &str, inventory: PlayerInventory) {
        let mut inventories = self.player_inventories.write().await;
        inventories.insert(player_id.to_string(), inventory);
    }

    async fn craft_item(&self, player_id: &str, recipe_name: &str, context: &mut PluginContext) -> Option<String> {
        let mut recipe_book = self.recipe_book.write().await;
        let mut player_inventory = self.get_player_inventory(player_id).await?;

        let mut inventory_map: HashMap<String, Ingredient> = player_inventory.slots.iter()
            .filter_map(|(_slot, item_opt)| item_opt.as_ref().map(|item| (item.name.clone(), Ingredient {
                name: item.name.clone(),
                quantity: 1,
                recipe_craftable: true,
            })))
            .collect();

        if let Some(crafted_item) = recipe_book.craft(recipe_name, &mut inventory_map).await {
            // Update player inventory
            for (_slot, item) in player_inventory.slots.iter_mut() {
                if let Some(inv_item) = item {
                    if let Some(ingredient) = inventory_map.get(&inv_item.name) {
                        if ingredient.quantity == 0 {
                            *item = None;
                        }
                    }
                }
            }

            // Add crafted item to inventory
            for (_slot, item) in player_inventory.slots.iter_mut() {
                if item.is_none() {
                    *item = Some(Item {
                        name: crafted_item.clone(),
                        model: None,
                        meta_tags: HashMap::new(),
                    });
                    break;
                }
            }

            self.update_player_inventory(player_id, player_inventory).await;

            // Emit custom events
            self.emit_custom_event(CustomEvent {
                event_type: "item_crafted".to_string(),
                data: Arc::new(crafted_item.clone()),
            }, context).await;

            self.emit_custom_event(CustomEvent {
                event_type: "inventory_changed".to_string(),
                data: Arc::new(player_id.to_string()),
            }, context).await;

            if recipe_book.get_recipe(recipe_name).map(|r| r.is_mastered()).unwrap_or(false) {
                self.emit_custom_event(CustomEvent {
                    event_type: "recipe_mastered".to_string(),
                    data: Arc::new(recipe_name.to_string()),
                }, context).await;
            }

            Some(crafted_item)
        } else {
            self.emit_custom_event(CustomEvent {
                event_type: "crafting_failed".to_string(),
                data: Arc::new(recipe_name.to_string()),
            }, context).await;
            None
        }
    }
}


#[async_trait]
impl BaseAPI for RecipeSmith {
    async fn on_game_event(&self, event: &GameEvent) {
        match event {
            GameEvent::PlayerJoined(player) => {
                println!("RecipeSmith: Player {} joined. Initializing crafting data...", player.id);
                self.create_player_inventory(&player.id, 20).await; // Assuming 20 inventory slots
            }
            GameEvent::Custom(custom_event) => {
                match custom_event.event_type.as_str() {
                    "recipe_learned" => println!("RecipeSmith: New recipe learned!"),
                    "item_crafted" => println!("RecipeSmith: Item crafted!"),
                    "inventory_changed" => println!("RecipeSmith: Inventory updated!"),
                    "recipe_mastered" => println!("RecipeSmith: Recipe mastered!"),
                    "crafting_failed" => println!("RecipeSmith: Crafting failed!"),
                    "storage_container_created" => println!("RecipeSmith: New storage container created!"),
                    "storage_container_accessed" => println!("RecipeSmith: Storage container accessed!"),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    async fn on_game_tick(&self, _delta_time: f64) {
        // Implement tick logic if needed
    }

    async fn register_custom_event(&self, event_type: &str, context: &mut PluginContext) {
        context.register_for_custom_event(event_type, Arc::new(self.clone())).await;
    }

    async fn emit_custom_event(&self, event: CustomEvent, context: &mut PluginContext) {
        context.dispatch_custom_event(event).await;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Plugin for RecipeSmith {
    fn on_load(&self) {
        println!("RecipeSmith plugin loaded");
    }

    fn on_unload(&self) {
        println!("RecipeSmith plugin unloaded");
    }

    fn execute(&self) {
        println!("RecipeSmith plugin executed");
    }

    fn initialize(&self, context: &mut PluginContext) {
        println!("RecipeSmith plugin initializing");
        let mut recipe_smith = self.clone();
        
        // Instead of spawning a new task, we'll call initialize_recipe_smith directly
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                recipe_smith.initialize_recipe_smith(context).await;
            });
        
        println!("RecipeSmith plugin initialized");
    }

    fn shutdown(&self, _context: &mut PluginContext) {
        println!("RecipeSmith plugin shut down");
    }

    fn on_enable(&self, _context: &mut PluginContext) {
        println!("RecipeSmith plugin enabled");
    }

    fn on_disable(&self, _context: &mut PluginContext) {
        println!("RecipeSmith plugin disabled");
    }
}

impl PluginInformation for RecipeSmith {
    fn name(&self) -> String {
        "RecipeSmith".to_string()
    }

    fn get_instance(&self) -> Box<dyn SayHello> {
        Box::new(self.clone())
    }
}

impl SayHello for RecipeSmith {
    fn say_hello(&self) -> String {
        "Hello from RecipeSmith! Ready to craft some amazing items?".to_string()
    }
}

pub fn create_plugin_metadata() -> RecipeSmith {
    RecipeSmith::new()
}

impl RecipeSmith {
    pub async fn get_all_recipes(&self) -> Vec<Recipe> {
        let recipe_book = self.recipe_book.read().await;
        recipe_book.recipes.values().cloned().collect()
    }

    pub async fn get_recipes_by_crafter(&self, crafter_name: &str) -> Vec<Recipe> {
        let recipe_book = self.recipe_book.read().await;
        let crafter = Crafter { name: crafter_name.to_string() };
        recipe_book.get_recipes_for_crafter(&crafter)
    }

    pub async fn add_new_recipe(&self, recipe: Recipe) {
        let mut recipe_book = self.recipe_book.write().await;
        recipe_book.add_recipe(recipe);
    }

    pub async fn get_player_inventory_contents(&self, player_id: &str) -> Option<Vec<Item>> {
        let inventory = self.get_player_inventory(player_id).await?;
        Some(inventory.slots.values().filter_map(|item| item.clone()).collect())
    }

    pub async fn add_item_to_player_inventory(&self, player_id: &str, item: Item) -> Result<(), String> {
        let mut inventory = self.get_player_inventory(player_id).await.ok_or("Player inventory not found")?;
        
        for (slot, item_opt) in inventory.slots.iter_mut() {
            if item_opt.is_none() {
                *item_opt = Some(item);
                self.get_player_inventory("player_id");
                return Ok(());
            }
        }
        
        Err("Inventory is full".to_string())
    }

    pub async fn remove_item_from_player_inventory(&self, player_id: &str, item_name: &str) -> Result<(), String> {
        let mut inventory = self.get_player_inventory(player_id).await.ok_or("Player inventory not found")?;
        
        for (slot, item_opt) in inventory.slots.iter_mut() {
            if let Some(item) = item_opt {
                if item.name == item_name {
                    *item_opt = None;
                    self.get_player_inventory("player_id");
                    return Ok(());
                }
            }
        }
        
        Err("Item not found in inventory".to_string())
    }

    pub async fn create_storage_container(&self, num_slots: u32) -> StorageContainer {
        StorageContainer::new(num_slots)
    }

    pub async fn access_storage_container(&self, container: &mut StorageContainer, player_id: &str, context: &mut PluginContext) {
        // Here you would implement the logic for a player accessing a storage container
        // For now, we'll just emit an event
        self.emit_custom_event(CustomEvent {
            event_type: "storage_container_accessed".to_string(),
            data: Arc::new((player_id.to_string(), container.uuid.to_string())),
        }, context).await;
    }

    pub async fn transfer_item(&self, from_inventory: &mut PlayerInventory, to_inventory: &mut PlayerInventory, item_name: &str) -> Result<(), String> {
        let mut item_to_transfer: Option<Item> = None;

        // Find and remove the item from the source inventory
        for (slot, item_opt) in from_inventory.slots.iter_mut() {
            if let Some(item) = item_opt {
                if item.name == item_name {
                    item_to_transfer = item_opt.take();
                    break;
                }
            }
        }

        // If we found the item, add it to the destination inventory
        if let Some(item) = item_to_transfer {
            for (slot, item_opt) in to_inventory.slots.iter_mut() {
                if item_opt.is_none() {
                    *item_opt = Some(item);
                    return Ok(());
                }
            }
            // If we couldn't add the item to the destination inventory, put it back in the source
            for (slot, item_opt) in from_inventory.slots.iter_mut() {
                if item_opt.is_none() {
                    *item_opt = Some(item);
                    break;
                }
            }
            Err("Destination inventory is full".to_string())
        } else {
            Err("Item not found in source inventory".to_string())
        }
    }
}
