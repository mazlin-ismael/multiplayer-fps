# 📦 Modèles 3D - Instructions de Téléchargement

## 🎯 Modèles Nécessaires

Vous avez besoin de 2 modèles :
1. **Soldat masqué** (player.glb)
2. **AK-47** (ak47.glb)

---

## 🚀 Option 1 : Téléchargement Rapide (Recommandé)

### Quaternius - Ultimate Animated Army Pack (GRATUIT)
Les meilleurs modèles low-poly pour FPS !

**Téléchargement :**
1. Visitez : https://quaternius.com/packs.html
2. Cherchez "Ultimate Animated Army Pack"
3. Cliquez sur "Download" (gratuit, aucune inscription requise)
4. Extrayez le ZIP

**Fichiers à copier :**
- Cherchez un soldat avec masque/casque dans le pack
- Renommez-le en `player.glb` et copiez dans ce dossier
- Cherchez une arme de type fusil d'assaut
- Renommez-la en `ak47.glb` et copiez dans ce dossier

---

## 🎨 Option 2 : Sketchfab (Plus de Choix)

### AK-47 (CS2 Style)
**Lien :** https://sketchfab.com/3d-models/rifle-ak-47-weapon-model-cs2-6b2244ba66274c71abdd194d0b04f731

**Instructions :**
1. Cliquez sur le lien
2. Cliquez sur "Download 3D Model" (gratuit)
3. Choisissez le format **glTF (.glb)**
4. Téléchargez et renommez en `ak47.glb`
5. Copiez dans ce dossier (`multiplayer-fps/assets/models/`)

### Soldat
**Recherche :** https://sketchfab.com/search?q=soldier+masked&type=models&features=downloadable

**Instructions :**
1. Cherchez un soldat masqué qui vous plaît
2. Vérifiez qu'il est "Downloadable" (icône de téléchargement)
3. Téléchargez en format **glTF (.glb)**
4. Renommez en `player.glb`
5. Copiez dans ce dossier

---

## 🔧 Option 3 : Autres Sources Gratuites

### Mixamo (Personnages Animés)
- **Lien :** https://www.mixamo.com/
- Nécessite un compte Adobe gratuit
- Personnages de haute qualité avec animations
- Format : FBX (convertissez en GLB avec Blender)

### CGTrader
- **AK-47 :** https://www.cgtrader.com/free-3d-models/military/gun/ak-47-7dee5417-9f78-472d-93b6-34d395f4d03b
- Format glTF disponible

---

## 📁 Structure Finale

Après téléchargement, votre dossier doit ressembler à :

```
multiplayer-fps/assets/models/
├── player.glb      (Soldat masqué)
├── ak47.glb        (AK-47)
└── README.md       (ce fichier)
```

---

## ✅ Vérification

Une fois les fichiers placés, lancez le jeu :

```bash
cd multiplayer-fps
cargo run --bin client
```

Vous devriez voir :
- ✅ Votre soldat masqué avec AK-47 visible depuis votre vue
- ✅ Les autres joueurs comme soldats masqués avec AK-47

---

## 🎮 Alternative Rapide : Modèles Basiques

Si vous voulez tester rapidement, je peux aussi créer des modèles procéduraux améliorés en attendant que vous trouviez les vrais skins.

**Besoin d'aide ?** Faites-moi savoir si vous avez des problèmes de téléchargement !
