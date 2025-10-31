use bevy::prelude::*;

// Marker component pour identifier le modèle GLTF du joueur
#[derive(Component)]
#[allow(dead_code)]
pub struct PlayerGltfModel;

// Marker component pour l'arme GLTF
#[derive(Component)]
#[allow(dead_code)]
pub struct WeaponGltfModel;

// Marker component pour identifier la tourelle du tank (qui tourne avec le yaw)
#[derive(Component)]
pub struct TankTurret;

// Marker component pour identifier le canon du tank (qui peut pivoter en hauteur)
#[derive(Component)]
pub struct TankCannon;

/// Crée un modèle 3D de TANK avec châssis, tourelle et canon
/// Le tank roule au sol (y=0) et le canon est bien visible
pub fn create_player_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Matériaux pour le tank
    let chassis_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.2), // Vert militaire pour le châssis
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..Default::default()
    });

    let turret_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.35, 0.25), // Vert légèrement plus clair
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..Default::default()
    });

    let cannon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15), // Gris foncé métallique
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..Default::default()
    });

    let track_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1), // Noir pour les chenilles
        metallic: 0.3,
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    // Meshes pour le tank
    let chassis_mesh = meshes.add(Cuboid::new(1.2, 0.4, 1.8)); // Châssis principal (large, bas, long)
    let turret_mesh = meshes.add(Cuboid::new(0.8, 0.5, 0.8)); // Tourelle sur le dessus
    let cannon_mesh = meshes.add(Cuboid::new(0.12, 0.12, 1.4)); // Canon long
    let track_left_mesh = meshes.add(Cuboid::new(0.15, 0.3, 1.8)); // Chenille gauche
    let track_right_mesh = meshes.add(Cuboid::new(0.15, 0.3, 1.8)); // Chenille droite

    // Entité parent pour le tank complet
    // Position 0,0,0 correspond à la position de la caméra (cockpit)
    let tank_model = commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .with_children(|parent| {
            // Châssis principal - au sol
            parent.spawn(PbrBundle {
                mesh: chassis_mesh,
                material: chassis_material.clone(),
                transform: Transform::from_xyz(0.0, -1.3, 0.0), // Bas, au niveau du sol
                ..Default::default()
            });

            // Chenille gauche
            parent.spawn(PbrBundle {
                mesh: track_left_mesh,
                material: track_material.clone(),
                transform: Transform::from_xyz(-0.65, -1.3, 0.0), // Côté gauche
                ..Default::default()
            });

            // Chenille droite
            parent.spawn(PbrBundle {
                mesh: track_right_mesh,
                material: track_material.clone(),
                transform: Transform::from_xyz(0.65, -1.3, 0.0), // Côté droit
                ..Default::default()
            });

            // TOURELLE - Entité parent qui tourne avec le yaw (rotation horizontale)
            // La tourelle et le canon tournent ensemble
            parent.spawn((
                SpatialBundle {
                    transform: Transform::from_xyz(0.0, -0.9, 0.0), // Sur le châssis
                    ..Default::default()
                },
                TankTurret, // Marker pour identifier et appliquer le yaw
            ))
            .with_children(|turret_parent| {
                // Mesh visuel de la tourelle
                turret_parent.spawn(PbrBundle {
                    mesh: turret_mesh,
                    material: turret_material.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0), // Position locale relative à la tourelle
                    ..Default::default()
                });

                // CANON - Long, pointant vers l'avant, bien visible !
                // Enfant de la tourelle, peut pivoter en hauteur (pitch) en plus du yaw hérité
                turret_parent.spawn((
                    PbrBundle {
                        mesh: cannon_mesh,
                        material: cannon_material.clone(),
                        transform: Transform::from_xyz(0.0, 0.05, 1.0), // Devant la tourelle, légèrement en haut
                        ..Default::default()
                    },
                    TankCannon, // Marker pour identifier et appliquer le pitch
                ));
            });
        })
        .id();

    tank_model
}

/// Crée un canon de tank visible en vue FPS
/// Le canon du tank est visible en bas de l'écran
pub fn create_fps_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let cannon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15), // Gris foncé métallique
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..Default::default()
    });

    let turret_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.2), // Vert militaire
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..Default::default()
    });

    // Meshes pour le canon de tank en vue FPS
    let cannon_mesh = meshes.add(Cuboid::new(0.1, 0.1, 1.2)); // Canon long et épais
    let cannon_base_mesh = meshes.add(Cuboid::new(0.15, 0.15, 0.2)); // Base du canon
    let turret_edge_mesh = meshes.add(Cuboid::new(0.5, 0.08, 0.3)); // Bord de la tourelle visible

    // Créer le canon FPS positionné en bas de la vue
    let fps_cannon = commands
        .spawn(SpatialBundle {
            // Position relative à la caméra
            // En bas au centre pour voir le canon pointer devant
            transform: Transform::from_xyz(0.0, -0.3, -0.4),
            ..Default::default()
        })
        .with_children(|parent| {
            // Canon principal - long, pointant vers l'avant
            parent.spawn(PbrBundle {
                mesh: cannon_mesh,
                material: cannon_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, -0.6), // Devant
                ..Default::default()
            });

            // Base du canon (sortie de la tourelle)
            parent.spawn(PbrBundle {
                mesh: cannon_base_mesh,
                material: turret_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.1), // Juste derrière
                ..Default::default()
            });

            // Bord de la tourelle visible en bas de l'écran
            parent.spawn(PbrBundle {
                mesh: turret_edge_mesh,
                material: turret_material,
                transform: Transform::from_xyz(0.0, -0.1, 0.2), // En bas, large
                ..Default::default()
            });
        })
        .id();

    fps_cannon
}

/// Crée un joueur avec modèle GLTF (soldat masqué)
/// Essaie de charger models/player.glb, sinon utilise le modèle procédural
#[allow(dead_code)]
pub fn create_player_model_gltf(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Essayer de charger le modèle GLTF (Forest Soldier)
    // Note: Bevy charge les assets de manière asynchrone, donc on crée toujours l'entité
    let gltf_handle: Handle<Scene> = asset_server.load("models/soldier.glb#Scene0");

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

    // Charger l'arme séparément pour la vue des autres joueurs (AK-47 de Sketchfab)
    let weapon_gltf: Handle<Scene> = asset_server.load("models/rifle.glb#Scene0");
    let weapon_entity = commands
        .spawn((
            SceneBundle {
                scene: weapon_gltf,
                // Position ajustée pour être tenue devant le soldat
                // Les valeurs sont relatives au soldat (qui est à y=0)
                transform: Transform::from_xyz(0.15, -0.4, 0.3) // Plus bas et plus près du corps
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2) // 90° rotation Y
                        * Quat::from_rotation_x(-0.2) // Légère inclinaison vers le bas
                    )
                    .with_scale(Vec3::splat(0.1)), // Échelle réduite pour éviter les "fils"
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
#[allow(dead_code)]
pub fn create_player_model_procedural(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Code procédural actuel (renommé pour clarté)
    create_player_model(commands, meshes, materials)
}
