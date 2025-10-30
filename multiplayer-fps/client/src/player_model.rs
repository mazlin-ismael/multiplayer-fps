use bevy::prelude::*;

/// Crée un modèle 3D de joueur composé de plusieurs parties
/// (tête, corps, bras, jambes) pour remplacer la simple capsule rouge
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

    // Meshes pour les parties du corps
    let head_mesh = meshes.add(Sphere::new(0.15));
    let body_mesh = meshes.add(Cuboid::new(0.4, 0.6, 0.25));
    let arm_mesh = meshes.add(Cuboid::new(0.12, 0.5, 0.12));
    let leg_mesh = meshes.add(Cuboid::new(0.15, 0.5, 0.15));

    // Entité parent pour le modèle complet
    let player_model = commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .with_children(|parent| {
            // Tête
            parent.spawn(PbrBundle {
                mesh: head_mesh,
                material: skin_material.clone(),
                transform: Transform::from_xyz(0.0, 0.65, 0.0),
                ..Default::default()
            });

            // Corps
            parent.spawn(PbrBundle {
                mesh: body_mesh,
                material: body_material.clone(),
                transform: Transform::from_xyz(0.0, 0.2, 0.0),
                ..Default::default()
            });

            // Bras gauche
            parent.spawn(PbrBundle {
                mesh: arm_mesh.clone(),
                material: limb_material.clone(),
                transform: Transform::from_xyz(-0.3, 0.25, 0.0),
                ..Default::default()
            });

            // Bras droit
            parent.spawn(PbrBundle {
                mesh: arm_mesh,
                material: limb_material.clone(),
                transform: Transform::from_xyz(0.3, 0.25, 0.0),
                ..Default::default()
            });

            // Jambe gauche
            parent.spawn(PbrBundle {
                mesh: leg_mesh.clone(),
                material: limb_material.clone(),
                transform: Transform::from_xyz(-0.12, -0.35, 0.0),
                ..Default::default()
            });

            // Jambe droite
            parent.spawn(PbrBundle {
                mesh: leg_mesh,
                material: limb_material.clone(),
                transform: Transform::from_xyz(0.12, -0.35, 0.0),
                ..Default::default()
            });
        })
        .id();

    player_model
}

/// Fonction alternative : crée un joueur avec un fichier GLTF
/// (pour usage futur si vous voulez importer des vrais modèles 3D)
pub fn create_player_model_from_gltf(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/player.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .id()
}
