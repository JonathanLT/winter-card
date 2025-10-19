# winter_card

Application web minimale avec Rocket (Rust) pour gérer des "access codes" via une interface admin.

## Structure du projet

Arborescence principale (vue synthétique)

```
winter_card/
├─ Cargo.toml
├─ winter_card.db                 # Base SQLite (générée au démarrage)
├─ src/
│  ├─ main.rs                     # point d'entrée : init DB, état et routes
│  ├─ db.rs                       # pool r2d2 + création / migrations simples
│  ├─ state.rs                    # AppState (pool DB + flag d'authentification)
│  ├─ auth.rs                     # request guard `AuthenticatedUser`
│  ├─ models/
│  │  ├─ mod.rs
│  │  └─ access_code.rs           # modèle AccessCode (id, code, active)
│  └─ routes/
│     ├─ mod.rs                   # regroupe et exporte toutes les routes
│     ├─ index.rs                 # routes publiques : /, /login, /logout
│     └─ admin.rs                 # routes admin : /admin + API codes
└─ README.md
```

Descriptions rapides
- src/main.rs : assemble Rocket, crée le pool SQLite, initialise la DB et partage AppState.
- src/db.rs : fonctions d'initialisation du pool et création des tables (users, access_codes, posts…).
- src/state.rs : définit AppState { db_pool, is_authenticated } et helpers éventuels.
- src/auth.rs : garde de requête pour protéger les routes admin/API.
- src/models/* : structures sérialisables (serde) pour AccessCode, User, Post.
- src/routes/* : handlers HTTP, pages HTML embarquées et API JSON (protégées par AuthenticatedUser).

## Dépendances principales

- rocket = "0.5.1" (feature "json")
- rusqlite, r2d2, r2d2_sqlite
- serde (derive)
- regex (validation côté serveur)

(Voir Cargo.toml pour la liste complète.)

## Lancer l'application (macOS / Linux)

1. Build & run :

   ```bash
   cargo build
   cargo run
   ```

2. Par défaut l'application écoute sur `http://localhost:8000`.

## Routes importantes

- GET  /                      → page d'accueil (login si non authentifié)
- POST /login                 → login (champ `password` contenant un access code actif)
- POST /logout                → logout
- GET  /admin                 → interface admin (protégée)
- GET  /admin/api/codes       → lister les access codes (JSON) — protégé
- POST /admin/api/codes       → créer un code (JSON { code: String, active: bool }) — protégé
- PATCH /admin/api/codes/<id> → mettre à jour `active` — protégé
- DELETE /admin/api/codes/<id>→ supprimer un code — protégé

## Remarques sur l'authentification

- Implémentation actuelle : flag global `is_authenticated` dans AppState + request guard.
- /login vérifie un code actif dans la table `access_codes`.
- Pour production : remplacer par sessions cookies/JWT et hachage des codes.

## Sécurité & améliorations recommandées

- Hacher les codes (argon2/bcrypt) — ne pas stocker en clair.
- Remplacer le flag global par une gestion de session (cookies signés, JWT ou DB).
- Activer HTTPS et améliorer la validation côté serveur.
- Ajouter tests unitaires et intégration pour routes et DB.
