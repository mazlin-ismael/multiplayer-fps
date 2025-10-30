# 🎯 GUIDE RAPIDE : Télécharger les Modèles 3D

Votre jeu est prêt à charger un **soldat masqué avec AK-47** !

## ⚡ Téléchargement Rapide (5 minutes)

### Étape 1 : Visitez Quaternius (Gratuit, Sans Inscription)

**Lien :** https://quaternius.com/packs.html

### Étape 2 : Téléchargez le Pack Gratuit

Cherchez ces packs (tous gratuits) :
- **"Ultimate Animated Army"** → Soldats avec casques/masques
- **"Ultimate Weapons"** → Fusils d'assaut, AK-47, etc.

Cliquez sur **"Download"** pour chaque pack

### Étape 3 : Extrayez les Fichiers

```bash
# Après téléchargement, extrayez les ZIP
unzip Ultimate_Animated_Army.zip
unzip Ultimate_Weapons.zip
```

### Étape 4 : Copiez les Modèles

Cherchez dans les dossiers extraits :
- Un fichier `.glb` de soldat avec masque/casque
- Un fichier `.glb` d'AK-47 ou fusil d'assaut

**Renommez et copiez :**

```bash
# Depuis le dossier du projet
cd multiplayer-fps/assets/models/

# Copiez vos fichiers ici et renommez-les :
# - Soldat → soldier.glb
# - AK-47 / Fusil → rifle.glb
```

### Étape 5 : Lancez le Jeu !

```bash
cd multiplayer-fps
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

---

## 🎨 Option Alternative : Sketchfab

Si vous préférez Sketchfab :

### Soldat Masqué
1. Allez sur : https://sketchfab.com/search?q=soldier+tactical+mask&type=models&features=downloadable
2. Choisissez un modèle **"Downloadable"** gratuit
3. Téléchargez en format **glTF Binary (.glb)** avec textures 1k
4. Renommez en `soldier.glb`

### AK-47
1. Allez sur : https://sketchfab.com/3d-models/rifle-ak-47-weapon-model-cs2-6b2244ba66274c71abdd194d0b04f731
2. Cliquez sur **"Download"**
3. Format **glTF Binary (.glb)** avec textures 1k
4. Renommez en `rifle.glb`

---

## 📁 Vérification Finale

Votre structure doit être :

```
multiplayer-fps/
└── assets/
    └── models/
        ├── soldier.glb   ← Soldat masqué (Forest Soldier)
        ├── rifle.glb     ← AK-47 / Fusil d'assaut
        └── README.md
```

---

## ⚠️ Si les Modèles Ne Se Chargent Pas

Le jeu utilisera automatiquement les **modèles procéduraux de fallback** (formes géométriques).

Pour vérifier que les modèles se chargent :
```bash
cargo run --bin client
# Regardez la console pour :
# "Loaded asset: models/soldier.glb"
# "Loaded asset: models/rifle.glb"
```

---

## 🔧 Ajuster la Taille des Modèles

Si les modèles sont trop grands/petits, éditez :

**`client/src/player_model.rs` ligne 263 :**
```rust
.with_scale(Vec3::splat(1.0)) // Changez 1.0 en 0.5 ou 2.0
```

**Pour l'arme, ligne 278 :**
```rust
.with_scale(Vec3::splat(0.8)) // Ajustez cette valeur
```

---

## ✅ C'est Tout !

Une fois les fichiers copiés, **redémarrez le jeu** et vous verrez des vrais soldats masqués avec AK-47 ! 🎮🔫
