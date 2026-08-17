//! Structured item data components.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind};

use crate::{
    Boolean, Float, IdOr, IdSet, Identifier, Int, Nbt, PrefixedArray, PrefixedOptional,
    PrefixedString, ProtocolEnum, SoundEvent, TextComponent, TypeCodec, TypeStructCodec, VarInt,
};

use super::{Slot, types::*};

macro_rules! component_struct {
    ($name:ident) => {
        #[doc = concat!("Wire payload for the `", stringify!($name), "` structured item component.")]
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = StructuredComponent)]
        pub struct $name;
    };
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[doc = concat!("Wire payload for the `", stringify!($name), "` structured item component.")]
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = StructuredComponent)]
        pub struct $name { $(pub $field: $ty,)* }
    };
}

component_struct!(CustomDataComponent { data: Nbt });
component_struct!(MaxStackSizeComponent {
    max_stack_size: VarInt
});
component_struct!(MaxDamageComponent { max_damage: VarInt });
component_struct!(DamageComponent { damage: VarInt });
component_struct!(UnbreakableComponent);
component_struct!(CustomNameComponent {
    name: TextComponent
});
component_struct!(ItemNameComponent {
    name: TextComponent
});
component_struct!(ItemModelComponent { model: Identifier });
component_struct!(LoreComponent { lines: PrefixedArray<TextComponent> });
component_struct!(RarityComponent { rarity: Rarity });
component_struct!(EnchantmentsComponent { enchantments: PrefixedArray<EnchantmentEntry> });
component_struct!(CanPlaceOnComponent { predicates: PrefixedArray<BlockPredicate> });
component_struct!(CanBreakComponent { predicates: PrefixedArray<BlockPredicate> });
component_struct!(AttributeModifiersComponent { modifiers: PrefixedArray<AttributeModifier> });
component_struct!(CustomModelDataComponent {
    floats: PrefixedArray<Float>, flags: PrefixedArray<Boolean>,
    strings: PrefixedArray<PrefixedString>, colors: PrefixedArray<Int>,
});
component_struct!(TooltipDisplayComponent {
    hide_tooltip: Boolean, hidden_components: PrefixedArray<DataComponentType>,
});
component_struct!(RepairCostComponent { cost: VarInt });
component_struct!(CreativeSlotLockComponent);
component_struct!(EnchantmentGlintOverrideComponent { has_glint: Boolean });
component_struct!(IntangibleProjectileComponent { empty: Nbt });
component_struct!(FoodComponent {
    nutrition: VarInt,
    saturation_modifier: Float,
    can_always_eat: Boolean,
});
component_struct!(ConsumableComponent {
    consume_seconds: Float, animation: ItemUseAnimation, sound: IdOr<SoundEvent>,
    has_consume_particles: Boolean, effects: PrefixedArray<ConsumeEffect>,
});
component_struct!(UseRemainderComponent { remainder: Box<Slot> });
component_struct!(UseCooldownComponent {
    seconds: Float, cooldown_group: PrefixedOptional<Identifier>,
});
component_struct!(UseEffectsComponent {
    can_sprint: Boolean,
    interact_vibrations: Boolean,
    speed_multiplier: Float,
});
component_struct!(MinimumAttackChargeComponent {
    minimum_attack_charge: Float
});
component_struct!(DamageTypeComponent {
    damage_type_id: VarInt
});
component_struct!(DamageResistantComponent { types: IdSet });
component_struct!(AttackRangeComponent {
    min_reach: Float,
    max_reach: Float,
    min_creative_reach: Float,
    max_creative_reach: Float,
    hitbox_margin: Float,
    mob_factor: Float,
});
component_struct!(ToolComponent {
    rules: PrefixedArray<ToolRule>, default_mining_speed: Float,
    damage_per_block: VarInt, can_destroy_blocks_in_creative: Boolean,
});
component_struct!(WeaponComponent {
    damage_per_attack: VarInt,
    disable_blocking_for: Float
});
component_struct!(EnchantableComponent { value: VarInt });
component_struct!(EquippableComponent {
    slot: EquipmentSlot, equip_sound: IdOr<SoundEvent>,
    model: PrefixedOptional<Identifier>, camera_overlay: PrefixedOptional<Identifier>,
    allowed_entities: PrefixedOptional<IdSet>, dispensable: Boolean, swappable: Boolean,
    damage_on_hurt: Boolean, can_be_sheared: Boolean, shearing_sound: IdOr<SoundEvent>,
});
component_struct!(RepairableComponent { items: IdSet });
component_struct!(GliderComponent);
component_struct!(TooltipStyleComponent { style: Identifier });
component_struct!(DeathProtectionComponent { effects: PrefixedArray<ConsumeEffect> });
component_struct!(BlocksAttacksComponent {
    block_delay_seconds: Float, disable_cooldown_scale: Float,
    damage_reductions: PrefixedArray<DamageReduction>, item_damage_threshold: Float,
    item_damage_base: Float, item_damage_factor: Float,
    bypassed_by: PrefixedOptional<IdSet>,
    block_sound: PrefixedOptional<IdOr<SoundEvent>>,
    disable_sound: PrefixedOptional<IdOr<SoundEvent>>,
});
component_struct!(StoredEnchantmentsComponent { enchantments: PrefixedArray<EnchantmentEntry> });
component_struct!(DyedColorComponent { color: Int });
component_struct!(MapColorComponent { color: Int });
component_struct!(MapIdComponent { id: VarInt });
component_struct!(MapDecorationsComponent { data: Nbt });
component_struct!(MapPostProcessingComponent {
    processing: MapPostProcessing
});
component_struct!(ChargedProjectilesComponent { projectiles: PrefixedArray<Slot> });
component_struct!(BundleContentsComponent { items: PrefixedArray<Slot> });
component_struct!(PotionContentsComponent {
    potion_id: PrefixedOptional<VarInt>, custom_color: PrefixedOptional<Int>,
    custom_effects: PrefixedArray<PotionEffect>, custom_name: PrefixedOptional<PrefixedString>,
});
component_struct!(PotionDurationScaleComponent {
    effect_multiplier: Float
});
component_struct!(SuspiciousStewEffectsComponent { effects: PrefixedArray<SuspiciousStewEffect> });
component_struct!(WritableBookContentComponent { pages: PrefixedArray<Filterable<PrefixedString>> });
component_struct!(WrittenBookContentComponent {
    title: Filterable<PrefixedString>, author: PrefixedString, generation: VarInt,
    pages: PrefixedArray<Filterable<TextComponent>>, resolved: Boolean,
});
component_struct!(TrimComponent {
    material: IdOr<TrimMaterial>, pattern: IdOr<TrimPattern>,
});
component_struct!(DebugStickStateComponent { data: Nbt });
component_struct!(EntityDataComponent {
    entity_type_id: VarInt,
    data: Nbt
});
component_struct!(BucketEntityDataComponent { data: Nbt });
component_struct!(BlockEntityDataComponent {
    block_entity_type_id: VarInt,
    data: Nbt
});
component_struct!(InstrumentComponent { instrument: IdOr<Instrument> });
component_struct!(PiercingWeaponComponent {
    deals_knockback: Boolean, dismounts: Boolean,
    sound: PrefixedOptional<SoundEvent>, hit_sound: PrefixedOptional<SoundEvent>,
});
component_struct!(KineticWeaponComponent {
    contact_cooldown_ticks: VarInt, delay_ticks: VarInt,
    dismount_conditions: PrefixedOptional<KineticWeaponCondition>,
    knockback_conditions: PrefixedOptional<KineticWeaponCondition>,
    damage_conditions: PrefixedOptional<KineticWeaponCondition>,
    forward_movement: Float, damage_multiplier: Float,
    sound: PrefixedOptional<SoundEvent>, hit_sound: PrefixedOptional<SoundEvent>,
});
component_struct!(SwingAnimationComponent {
    animation_type: SwingAnimationType,
    duration: VarInt
});
component_struct!(AdditionalTradeCostComponent {
    additional_trade_cost: VarInt
});
component_struct!(DyeComponent { color: DyeColor });
component_struct!(ProvidesTrimMaterialComponent { material: IdOr<TrimMaterial> });
component_struct!(OminousBottleAmplifierComponent { amplifier: VarInt });
component_struct!(JukeboxPlayableComponent { song: IdOr<JukeboxSong> });
component_struct!(ProvidesBannerPatternsComponent { patterns: IdSet });
component_struct!(RecipesComponent { data: Nbt });
component_struct!(LodestoneTrackerComponent {
    target: PrefixedOptional<GlobalPosition>, tracked: Boolean,
});
component_struct!(FireworkExplosionComponent {
    explosion: FireworkExplosion
});
component_struct!(FireworksComponent {
    flight_duration: VarInt, explosions: PrefixedArray<FireworkExplosion>,
});
component_struct!(ProfileComponent {
    profile: ResolvableProfile
});
component_struct!(NoteBlockSoundComponent { sound: Identifier });
component_struct!(BannerPatternsComponent { layers: PrefixedArray<BannerPatternLayer> });
component_struct!(BaseColorComponent { color: DyeColor });
component_struct!(PotDecorationsComponent { decorations: PrefixedArray<VarInt> });
component_struct!(ContainerComponent { items: PrefixedArray<Slot> });
component_struct!(BlockStateComponent { properties: PrefixedArray<BlockStateProperty> });
component_struct!(BeesComponent { bees: PrefixedArray<Bee> });
component_struct!(LockComponent { key: NbtString });
component_struct!(ContainerLootComponent { data: Nbt });
component_struct!(BreakSoundComponent { sound: IdOr<SoundEvent> });
component_struct!(SulfurCubeContentComponent { content: Box<Slot> });

component_struct!(VillagerVariantComponent { variant_id: VarInt });
component_struct!(WolfVariantComponent { variant_id: VarInt });
component_struct!(WolfSoundVariantComponent { variant_id: VarInt });
component_struct!(WolfCollarComponent { color: DyeColor });
component_struct!(FoxVariantComponent {
    variant: FoxVariant
});
component_struct!(SalmonSizeComponent { size: SalmonSize });
component_struct!(ParrotVariantComponent {
    variant: ParrotVariant
});
component_struct!(TropicalFishPatternComponent {
    pattern: TropicalFishPattern
});
component_struct!(TropicalFishBaseColorComponent { color: DyeColor });
component_struct!(TropicalFishPatternColorComponent { color: DyeColor });
component_struct!(MooshroomVariantComponent {
    variant: MooshroomVariant
});
component_struct!(RabbitVariantComponent {
    variant: RabbitVariant
});
component_struct!(PigVariantComponent { variant_id: VarInt });
component_struct!(PigSoundVariantComponent { variant_id: VarInt });
component_struct!(CowVariantComponent { variant_id: VarInt });
component_struct!(CowSoundVariantComponent { variant_id: VarInt });
component_struct!(ChickenVariantComponent { variant_id: VarInt });
component_struct!(ChickenSoundVariantComponent { variant_id: VarInt });
component_struct!(FrogVariantComponent { variant_id: VarInt });
component_struct!(HorseVariantComponent {
    variant: HorseVariant
});
component_struct!(PaintingVariantComponent { variant: IdOr<PaintingVariant> });
component_struct!(LlamaVariantComponent {
    variant: LlamaVariant
});
component_struct!(AxolotlVariantComponent {
    variant: AxolotlVariant
});
component_struct!(ZombieNautilusVariantComponent { variant_id: VarInt });
component_struct!(CatVariantComponent { variant_id: VarInt });
component_struct!(CatSoundVariantComponent { variant_id: VarInt });
component_struct!(CatCollarComponent { color: DyeColor });
component_struct!(SheepColorComponent { color: DyeColor });
component_struct!(ShulkerColorComponent { color: DyeColor });

macro_rules! small_enum {
    ($name:ident { $($variant:ident = $id:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
        #[protocol_enum(repr = VarInt)]
        pub enum $name { $($variant = $id,)+ }
    };
}

small_enum!(FoxVariant { Red = 0, Snow = 1 });
small_enum!(SalmonSize { Small = 0, Medium = 1, Large = 2 });
small_enum!(ParrotVariant { RedBlue = 0, Blue = 1, Green = 2, YellowBlue = 3, Gray = 4 });
small_enum!(TropicalFishPattern {
    Kob = 0, Sunstreak = 1, Snooper = 2, Dasher = 3, Brinely = 4, Spotty = 5,
    Flopper = 6, Stripey = 7, Glitter = 8, Blockfish = 9, Betty = 10, Clayfish = 11,
});
small_enum!(MooshroomVariant { Red = 0, Brown = 1 });
small_enum!(RabbitVariant {
    Brown = 0, White = 1, Black = 2, WhiteSplotched = 3, Gold = 4, Salt = 5, Evil = 6,
});
small_enum!(HorseVariant {
    White = 0, Creamy = 1, Chestnut = 2, Brown = 3, Black = 4, Gray = 5, DarkBrown = 6,
});
small_enum!(LlamaVariant { Creamy = 0, White = 1, Brown = 2, Gray = 3 });
small_enum!(AxolotlVariant { Lucy = 0, Wild = 1, Gold = 2, Cyan = 3, Blue = 4 });

macro_rules! data_components {
    ($($variant:ident($payload:ty) = $id:literal => $key:literal),+ $(,)?) => {
        /// Numeric IDs in the current `minecraft:data_component_type` registry.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
        #[protocol_enum(repr = VarInt)]
        pub enum DataComponentType { $($variant = $id,)+ }

        impl DataComponentType {
            /// Returns the namespaced registry key documented for this type.
            pub const fn identifier(self) -> &'static str {
                match self { $(Self::$variant => $key,)+ }
            }
        }

        /// A component type ID paired with exactly the payload required by that ID.
        #[derive(Debug, Clone, PartialEq)]
        pub enum DataComponent { $($variant($payload),)+ }

        impl DataComponent {
            /// Returns this value's component type.
            pub const fn component_type(&self) -> DataComponentType {
                match self { $(Self::$variant(_) => DataComponentType::$variant,)+ }
            }
        }

        impl TypeCodec for DataComponent {
            fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
                self.component_type().encode(writer)
                    .map_err(|e| e.with_context(CodecKind::DataComponent))?;
                match self { $(Self::$variant(value) => value.encode(writer),)+ }
                    .map_err(|e| e.with_context(CodecKind::DataComponent))
            }
            fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
                let ty = DataComponentType::decode(reader)
                    .map_err(|e| e.with_context(CodecKind::DataComponent))?;
                match ty { $(DataComponentType::$variant => <$payload>::decode(reader).map(Self::$variant),)+ }
                    .map_err(|e| e.with_context(CodecKind::DataComponent))
            }
        }
    };
}

data_components! {
    CustomData(CustomDataComponent)=0=>"minecraft:custom_data",
    MaxStackSize(MaxStackSizeComponent)=1=>"minecraft:max_stack_size",
    MaxDamage(MaxDamageComponent)=2=>"minecraft:max_damage",
    Damage(DamageComponent)=3=>"minecraft:damage",
    Unbreakable(UnbreakableComponent)=4=>"minecraft:unbreakable",
    CustomName(CustomNameComponent)=5=>"minecraft:custom_name",
    ItemName(ItemNameComponent)=6=>"minecraft:item_name",
    ItemModel(ItemModelComponent)=7=>"minecraft:item_model",
    Lore(LoreComponent)=8=>"minecraft:lore",
    Rarity(RarityComponent)=9=>"minecraft:rarity",
    Enchantments(EnchantmentsComponent)=10=>"minecraft:enchantments",
    CanPlaceOn(CanPlaceOnComponent)=11=>"minecraft:can_place_on",
    CanBreak(CanBreakComponent)=12=>"minecraft:can_break",
    AttributeModifiers(AttributeModifiersComponent)=13=>"minecraft:attribute_modifiers",
    CustomModelData(CustomModelDataComponent)=14=>"minecraft:custom_model_data",
    TooltipDisplay(TooltipDisplayComponent)=15=>"minecraft:tooltip_display",
    RepairCost(RepairCostComponent)=16=>"minecraft:repair_cost",
    CreativeSlotLock(CreativeSlotLockComponent)=17=>"minecraft:creative_slot_lock",
    EnchantmentGlintOverride(EnchantmentGlintOverrideComponent)=18=>"minecraft:enchantment_glint_override",
    IntangibleProjectile(IntangibleProjectileComponent)=19=>"minecraft:intangible_projectile",
    Food(FoodComponent)=20=>"minecraft:food",
    Consumable(ConsumableComponent)=21=>"minecraft:consumable",
    UseRemainder(UseRemainderComponent)=22=>"minecraft:use_remainder",
    UseCooldown(UseCooldownComponent)=23=>"minecraft:use_cooldown",
    UseEffects(UseEffectsComponent)=24=>"minecraft:use_effects",
    MinimumAttackCharge(MinimumAttackChargeComponent)=25=>"minecraft:minimum_attack_charge",
    DamageType(DamageTypeComponent)=26=>"minecraft:damage_type",
    DamageResistant(DamageResistantComponent)=27=>"minecraft:damage_resistant",
    AttackRange(AttackRangeComponent)=28=>"minecraft:attack_range",
    Tool(ToolComponent)=29=>"minecraft:tool",
    Weapon(WeaponComponent)=30=>"minecraft:weapon",
    Enchantable(EnchantableComponent)=31=>"minecraft:enchantable",
    Equippable(EquippableComponent)=32=>"minecraft:equippable",
    Repairable(RepairableComponent)=33=>"minecraft:repairable",
    Glider(GliderComponent)=34=>"minecraft:glider",
    TooltipStyle(TooltipStyleComponent)=35=>"minecraft:tooltip_style",
    DeathProtection(DeathProtectionComponent)=36=>"minecraft:death_protection",
    BlocksAttacks(BlocksAttacksComponent)=37=>"minecraft:blocks_attacks",
    StoredEnchantments(StoredEnchantmentsComponent)=38=>"minecraft:stored_enchantments",
    DyedColor(DyedColorComponent)=39=>"minecraft:dyed_color",
    MapColor(MapColorComponent)=40=>"minecraft:map_color",
    MapId(MapIdComponent)=41=>"minecraft:map_id",
    MapDecorations(MapDecorationsComponent)=42=>"minecraft:map_decorations",
    MapPostProcessing(MapPostProcessingComponent)=43=>"minecraft:map_post_processing",
    ChargedProjectiles(ChargedProjectilesComponent)=44=>"minecraft:charged_projectiles",
    BundleContents(BundleContentsComponent)=45=>"minecraft:bundle_contents",
    PotionContents(PotionContentsComponent)=46=>"minecraft:potion_contents",
    PotionDurationScale(PotionDurationScaleComponent)=47=>"minecraft:potion_duration_scale",
    SuspiciousStewEffects(SuspiciousStewEffectsComponent)=48=>"minecraft:suspicious_stew_effects",
    WritableBookContent(WritableBookContentComponent)=49=>"minecraft:writable_book_content",
    WrittenBookContent(WrittenBookContentComponent)=50=>"minecraft:written_book_content",
    Trim(TrimComponent)=51=>"minecraft:trim",
    DebugStickState(DebugStickStateComponent)=52=>"minecraft:debug_stick_state",
    EntityData(EntityDataComponent)=53=>"minecraft:entity_data",
    BucketEntityData(BucketEntityDataComponent)=54=>"minecraft:bucket_entity_data",
    BlockEntityData(BlockEntityDataComponent)=55=>"minecraft:block_entity_data",
    Instrument(InstrumentComponent)=56=>"minecraft:instrument",
    PiercingWeapon(PiercingWeaponComponent)=57=>"minecraft:piercing_weapon",
    KineticWeapon(KineticWeaponComponent)=58=>"minecraft:kinetic_weapon",
    SwingAnimation(SwingAnimationComponent)=59=>"minecraft:swing_animation",
    AdditionalTradeCost(AdditionalTradeCostComponent)=60=>"minecraft:additional_trade_cost",
    Dye(DyeComponent)=61=>"minecraft:dye",
    ProvidesTrimMaterial(ProvidesTrimMaterialComponent)=62=>"minecraft:provides_trim_material",
    OminousBottleAmplifier(OminousBottleAmplifierComponent)=63=>"minecraft:ominous_bottle_amplifier",
    JukeboxPlayable(JukeboxPlayableComponent)=64=>"minecraft:jukebox_playable",
    ProvidesBannerPatterns(ProvidesBannerPatternsComponent)=65=>"minecraft:provides_banner_patterns",
    Recipes(RecipesComponent)=66=>"minecraft:recipes",
    LodestoneTracker(LodestoneTrackerComponent)=67=>"minecraft:lodestone_tracker",
    FireworkExplosion(FireworkExplosionComponent)=68=>"minecraft:firework_explosion",
    Fireworks(FireworksComponent)=69=>"minecraft:fireworks",
    Profile(ProfileComponent)=70=>"minecraft:profile",
    NoteBlockSound(NoteBlockSoundComponent)=71=>"minecraft:note_block_sound",
    BannerPatterns(BannerPatternsComponent)=72=>"minecraft:banner_patterns",
    BaseColor(BaseColorComponent)=73=>"minecraft:base_color",
    PotDecorations(PotDecorationsComponent)=74=>"minecraft:pot_decorations",
    Container(ContainerComponent)=75=>"minecraft:container",
    BlockState(BlockStateComponent)=76=>"minecraft:block_state",
    Bees(BeesComponent)=77=>"minecraft:bees",
    Lock(LockComponent)=78=>"minecraft:lock",
    ContainerLoot(ContainerLootComponent)=79=>"minecraft:container_loot",
    BreakSound(BreakSoundComponent)=80=>"minecraft:break_sound",
    SulfurCubeContent(SulfurCubeContentComponent)=81=>"minecraft:sulfur_cube_content",
    VillagerVariant(VillagerVariantComponent)=82=>"minecraft:villager/variant",
    WolfVariant(WolfVariantComponent)=83=>"minecraft:wolf/variant",
    WolfSoundVariant(WolfSoundVariantComponent)=84=>"minecraft:wolf/sound_variant",
    WolfCollar(WolfCollarComponent)=85=>"minecraft:wolf/collar",
    FoxVariant(FoxVariantComponent)=86=>"minecraft:fox/variant",
    SalmonSize(SalmonSizeComponent)=87=>"minecraft:salmon/size",
    ParrotVariant(ParrotVariantComponent)=88=>"minecraft:parrot/variant",
    TropicalFishPattern(TropicalFishPatternComponent)=89=>"minecraft:tropical_fish/pattern",
    TropicalFishBaseColor(TropicalFishBaseColorComponent)=90=>"minecraft:tropical_fish/base_color",
    TropicalFishPatternColor(TropicalFishPatternColorComponent)=91=>"minecraft:tropical_fish/pattern_color",
    MooshroomVariant(MooshroomVariantComponent)=92=>"minecraft:mooshroom/variant",
    RabbitVariant(RabbitVariantComponent)=93=>"minecraft:rabbit/variant",
    PigVariant(PigVariantComponent)=94=>"minecraft:pig/variant",
    PigSoundVariant(PigSoundVariantComponent)=95=>"minecraft:pig/sound_variant",
    CowVariant(CowVariantComponent)=96=>"minecraft:cow/variant",
    CowSoundVariant(CowSoundVariantComponent)=97=>"minecraft:cow/sound_variant",
    ChickenVariant(ChickenVariantComponent)=98=>"minecraft:chicken/variant",
    ChickenSoundVariant(ChickenSoundVariantComponent)=99=>"minecraft:chicken/sound_variant",
    FrogVariant(FrogVariantComponent)=100=>"minecraft:frog/variant",
    HorseVariant(HorseVariantComponent)=101=>"minecraft:horse/variant",
    PaintingVariant(PaintingVariantComponent)=102=>"minecraft:painting/variant",
    LlamaVariant(LlamaVariantComponent)=103=>"minecraft:llama/variant",
    AxolotlVariant(AxolotlVariantComponent)=104=>"minecraft:axolotl/variant",
    ZombieNautilusVariant(ZombieNautilusVariantComponent)=105=>"minecraft:zombie_nautilus/variant",
    CatVariant(CatVariantComponent)=106=>"minecraft:cat/variant",
    CatSoundVariant(CatSoundVariantComponent)=107=>"minecraft:cat/sound_variant",
    CatCollar(CatCollarComponent)=108=>"minecraft:cat/collar",
    SheepColor(SheepColorComponent)=109=>"minecraft:sheep/color",
    ShulkerColor(ShulkerColorComponent)=110=>"minecraft:shulker/color",
}
