# Modèles 3D des Joueurs

## Changements Implémentés

Les cylindres/capsules rouges représentant les autres joueurs ont été remplacés par des modèles 3D composés de plusieurs parties :

### Structure du Modèle Procédural

Le modèle actuel est créé programmatiquement dans `client/src/player_model.rs` et comprend :

1. **Tête** - Sphère avec texture couleur peau (rayon: 0.15)
2. **Corps** - Cuboid bleu (0.4 × 0.6 × 0.25)
3. **Bras gauche** - Cuboid bleu foncé (0.12 × 0.5 × 0.12)
4. **Bras droit** - Cuboid bleu foncé (0.12 × 0.5 × 0.12)
5. **Jambe gauche** - Cuboid bleu foncé (0.15 × 0.5 × 0.15)
6. **Jambe droite** - Cuboid bleu foncé (0.15 × 0.5 × 0.15)

### Fichiers Modifiés

- `client/src/player_model.rs` - **NOUVEAU** : Module de création des modèles 3D
- `client/src/other_players.rs` - Remplace la création de capsule par `create_player_model()`
- `client/src/main.rs` - Ajout du module `player_model`

### Apparence

Le modèle stylisé ressemble à un personnage "low-poly" avec :
- Couleurs distinctives (bleu pour le corps, teinte peau pour la tête)
- Matériaux PBR (metallic + roughness pour un rendu réaliste)
- Hiérarchie parent-enfant (le modèle entier se déplace comme une unité)

## Étendre avec des Modèles GLTF/GLB

### Option 1 : Utiliser un Modèle GLTF Personnalisé

Le fichier `player_model.rs` contient déjà une fonction `create_player_model_from_gltf()` pour charger des modèles externes.

**Étapes pour utiliser un modèle GLTF :**

1. Placez votre fichier `player.glb` dans `multiplayer-fps/assets/models/`

2. Modifiez `other_players.rs` ligne 36 :
   ```rust
   // Remplacez :
   let player_model = create_player_model(&mut commands, &mut meshes, &mut materials);

   // Par :
   let player_model = create_player_model_from_gltf(&mut commands, &asset_server);
   ```

3. Ajoutez le paramètre `asset_server` dans la signature de `receive_other_players_system` :
   ```rust
   pub fn receive_other_players_system(
       mut client: ResMut<RenetClient>,
       mut commands: Commands,
       mut other_players: ResMut<OtherPlayers>,
       asset_server: Res<AssetServer>, // AJOUTEZ CETTE LIGNE
       mut meshes: ResMut<Assets<Mesh>>,
       mut materials: ResMut<Assets<StandardMaterial>>,
       query: Query<(Entity, &OtherPlayer)>,
   )
   ```

### Option 2 : Modèles Multiples (Skins différents)

Pour ajouter plusieurs skins de joueurs :

```rust
pub fn create_player_model_with_variant(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    variant: u8, // 0 = bleu, 1 = rouge, 2 = vert, etc.
) -> Entity {
    let body_color = match variant {
        0 => Color::srgb(0.2, 0.4, 0.8), // Bleu
        1 => Color::srgb(0.8, 0.2, 0.2), // Rouge
        2 => Color::srgb(0.2, 0.8, 0.2), // Vert
        _ => Color::srgb(0.8, 0.8, 0.2), // Jaune
    };

    // ... reste du code avec body_color
}
```

### Ressources de Modèles 3D Gratuits

- **Mixamo** - https://www.mixamo.com/ (personnages animés)
- **Sketchfab** - https://sketchfab.com/feed (recherchez "CC0" ou "Creative Commons")
- **Quaternius** - https://quaternius.com/ (assets low-poly gratuits)
- **Kenney** - https://kenney.nl/assets (assets de jeu gratuits)

### Animations (Futur)

Pour ajouter des animations de marche/course, vous devrez :
1. Utiliser des modèles GLTF avec animations intégrées
2. Ajouter le plugin Bevy d'animation
3. Créer un système pour déclencher les animations basées sur la vélocité

## Notes Techniques

- **Hauteur du modèle** : ~1.7m (correspondant à la capsule de collision du joueur)
- **Pivot** : Au centre du modèle (0, 0, 0) dans le SpatialBundle parent
- **Éclairage** : Compatible avec les lumières PBR de la scène (point light + directional light)
- **Performance** : Le modèle procédural est très léger (6 meshes simples par joueur)

## Tests

Pour tester :
1. Compilez et lancez le serveur : `cargo run --bin server`
2. Lancez 2+ clients : `cargo run --bin client`
3. Connectez-vous à `127.0.0.1:5000`
4. Déplacez-vous pour voir les autres joueurs avec leurs nouveaux modèles 3D

Les joueurs apparaissent maintenant comme des personnages stylisés au lieu de simples cylindres rouges !
