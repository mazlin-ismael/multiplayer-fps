use bevy::prelude::*;

// Marker component pour identifier le modèle GLTF du joueur
#[derive(Component)]
pub struct PlayerGltfModel;

// Marker component pour l'arme GLTF
#[derive(Component)]
pub struct WeaponGltfModel;

/// Crée un modèle 3D de joueur composé de plusieurs parties
/// (tête, corps, bras, jambes) pour remplacer la simple capsule rouge
/// Le modèle est positionné pour que les pieds touchent le sol (y=0) quand
/// la position donnée est la position de la caméra (y=1.7)
pub fn create_player_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Matériaux pour les différentes parties du corps
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.4, 0.8), // Bleu pour le corps
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    let skin_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.8, 0.7), // Couleur peau pour la tête
        metallic: 0.0,
        perceptual_roughness: 0.9,
        ..Default::default()
    });

    let limb_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.35, 0.7), // Bleu foncé pour les membres
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    let weapon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1), // Noir pour l'arme
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..Default::default()
    });

    // Meshes pour les parties du corps
    let head_mesh = meshes.add(Sphere::new(0.15));
    let body_mesh = meshes.add(Cuboid::new(0.4, 0.6, 0.25));
    let arm_mesh = meshes.add(Cuboid::new(0.12, 0.5, 0.12));
    let leg_mesh = meshes.add(Cuboid::new(0.15, 0.5, 0.15));

    // Arme (fusil d'assaut) - Plus grande et visible
    let weapon_body_mesh = meshes.add(Cuboid::new(0.1, 0.15, 0.7)); // Corps du fusil
    let weapon_barrel_mesh = meshes.add(Cuboid::new(0.06, 0.06, 0.5)); // Canon long
    let weapon_stock_mesh = meshes.add(Cuboid::new(0.08, 0.1, 0.25)); // Crosse
    let weapon_grip_mesh = meshes.add(Cuboid::new(0.07, 0.15, 0.08)); // Poignée

    // Entité parent pour le modèle complet
    // Position 0,0,0 correspond à la position de la caméra (yeux à y=1.7)
    let player_model = commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .with_children(|parent| {
            // Tête (au niveau des yeux, légèrement au-dessus)
            parent.spawn(PbrBundle {
                mesh: head_mesh,
                material: skin_material.clone(),
                transform: Transform::from_xyz(0.0, -0.2, 0.0),
                ..Default::default()
            });

            // Corps (centré entre épaules et hanches)
            parent.spawn(PbrBundle {
                mesh: body_mesh,
                material: body_material.clone(),
                transform: Transform::from_xyz(0.0, -0.8, 0.0),
                ..Default::default()
            });

            // Bras gauche (au niveau des épaules)
            parent.spawn(PbrBundle {
                mesh: arm_mesh.clone(),
                material: limb_material.clone(),
                transform: Transform::from_xyz(-0.3, -0.75, 0.0),
                ..Default::default()
            });

            // Bras droit (avec arme)
            parent.spawn(PbrBundle {
                mesh: arm_mesh,
                material: limb_material.clone(),
                transform: Transform::from_xyz(0.3, -0.75, 0.0),
                ..Default::default()
            });

            // Jambe gauche
            parent.spawn(PbrBundle {
                mesh: leg_mesh.clone(),
                material: limb_material.clone(),
                transform: Transform::from_xyz(-0.12, -1.45, 0.0),
                ..Default::default()
            });

            // Jambe droite
            parent.spawn(PbrBundle {
                mesh: leg_mesh,
                material: limb_material.clone(),
                transform: Transform::from_xyz(0.12, -1.45, 0.0),
                ..Default::default()
            });

            // Arme (fusil d'assaut) tenue en diagonale devant le joueur
            // Corps principal du fusil
            parent.spawn(PbrBundle {
                mesh: weapon_body_mesh,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.22, -0.55, 0.4)
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_4 * 0.3) // Légère rotation
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 * 0.5) // Incliné vers le bas
                    ),
                ..Default::default()
            });

            // Canon long pointant vers l'avant
            parent.spawn(PbrBundle {
                mesh: weapon_barrel_mesh,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.22, -0.5, 0.75)
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_4 * 0.3)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 * 0.5)
                    ),
                ..Default::default()
            });

            // Crosse du fusil (derrière)
            parent.spawn(PbrBundle {
                mesh: weapon_stock_mesh,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.18, -0.58, 0.1)
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_4 * 0.3)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 * 0.5)
                    ),
                ..Default::default()
            });

            // Poignée (sous le corps)
            parent.spawn(PbrBundle {
                mesh: weapon_grip_mesh,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.2, -0.65, 0.35)
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_4 * 0.3)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 * 0.5)
                    ),
                ..Default::default()
            });
        })
        .id();

    player_model
}

/// Crée une arme FPS (vue première personne) attachée à la caméra
/// L'arme est visible dans le coin bas-droit de l'écran comme dans un FPS classique
pub fn create_fps_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let weapon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15), // Gris foncé
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..Default::default()
    });

    let handle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.2, 0.1), // Marron pour la crosse
        metallic: 0.0,
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    // Meshes pour le fusil FPS
    let weapon_body = meshes.add(Cuboid::new(0.08, 0.12, 0.6)); // Corps principal
    let weapon_barrel = meshes.add(Cuboid::new(0.04, 0.04, 0.4)); // Canon
    let weapon_handle = meshes.add(Cuboid::new(0.06, 0.15, 0.08)); // Poignée
    let weapon_stock = meshes.add(Cuboid::new(0.06, 0.08, 0.2)); // Crosse

    // Créer l'arme FPS positionnée dans le coin bas-droit de la vue
    let fps_weapon = commands
        .spawn(SpatialBundle {
            // Position relative à la caméra
            // X positif = droite, Y négatif = bas, Z négatif = devant la caméra
            transform: Transform::from_xyz(0.3, -0.25, -0.5),
            ..Default::default()
        })
        .with_children(|parent| {
            // Corps du fusil
            parent.spawn(PbrBundle {
                mesh: weapon_body,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, -0.1),
                ..Default::default()
            });

            // Canon (pointant vers l'avant)
            parent.spawn(PbrBundle {
                mesh: weapon_barrel,
                material: weapon_material.clone(),
                transform: Transform::from_xyz(0.0, 0.03, -0.5),
                ..Default::default()
            });

            // Poignée
            parent.spawn(PbrBundle {
                mesh: weapon_handle,
                material: handle_material.clone(),
                transform: Transform::from_xyz(0.0, -0.1, 0.05),
                ..Default::default()
            });

            // Crosse
            parent.spawn(PbrBundle {
                mesh: weapon_stock,
                material: handle_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.25)
                    .with_rotation(Quat::from_rotation_x(-0.3)),
                ..Default::default()
            });
        })
        .id();

    fps_weapon
}

/// Crée un joueur avec modèle GLTF (soldat masqué)
/// Essaie de charger models/player.glb, sinon utilise le modèle procédural
pub fn create_player_model_gltf(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Essayer de charger le modèle GLTF
    // Note: Bevy charge les assets de manière asynchrone, donc on crée toujours l'entité
    let gltf_handle: Handle<Scene> = asset_server.load("models/player.glb#Scene0");

    // Créer l'entité parent
    let player_entity = commands
        .spawn((
            SceneBundle {
                scene: gltf_handle,
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::splat(1.0)), // Ajustez l'échelle si nécessaire
                ..Default::default()
            },
            PlayerGltfModel,
        ))
        .id();

    // Charger l'arme séparément pour la vue des autres joueurs
    let weapon_gltf: Handle<Scene> = asset_server.load("models/ak47.glb#Scene0");
    let weapon_entity = commands
        .spawn((
            SceneBundle {
                scene: weapon_gltf,
                transform: Transform::from_xyz(0.3, -0.5, 0.4)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4 * 0.3))
                    .with_scale(Vec3::splat(0.8)), // Ajustez selon la taille de votre modèle
                ..Default::default()
            },
            WeaponGltfModel,
        ))
        .id();

    // Attacher l'arme au joueur
    commands.entity(player_entity).add_child(weapon_entity);

    player_entity
}

/// Version procédurale de fallback si les modèles GLTF ne sont pas disponibles
/// (garde le code actuel pour compatibilité)
pub fn create_player_model_procedural(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Code procédural actuel (renommé pour clarté)
    create_player_model(commands, meshes, materials)
}
