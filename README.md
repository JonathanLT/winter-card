# winter_card

Application web minimale avec Rocket (Rust) pour gérer un secret santa. Une interface admin permet d'ajouter des codes d'accès nommées.

## Description

Application simple pour gérer des access codes stockés dans SQLite. Permet l'authentification via un code actif et expose une interface admin pour gérer les codes (CRUD). Lors de l'authentification, l'AccessCode complet est conservé en mémoire dans l'état de l'application pour réutilisation.

## Structure du projet

Arborescence principale (vue synthétique)

```plain
winter_card/
├─ Cargo.toml
├─ winter_card.db                 # Base SQLite (générée / mise à jour au démarrage)
├─ src/
│  ├─ main.rs                     # point d'entrée : init DB, état et routes
│  ├─ db.rs                       # pool r2d2 + création / migrations simples
│  ├─ state.rs                    # AppState (pool DB + flags + accès courant)
│  ├─ auth.rs                     # request guard `AuthenticatedUser`
│  ├─ models/
│  │  ├─ mod.rs
│  │  ├─ access_code.rs          # modèle AccessCode (id, name, code, active)
│  │  └─ draw.rs                 # modèle Draw pour le Secret Santa
│  └─ routes/
│     ├─ mod.rs                  # regroupe et exporte toutes les routes
│     ├─ index.rs                # routes publiques : /, /login, /logout
│     ├─ admin.rs                # routes admin : /admin + API codes
│     └─ secret_santa.rs         # route publique /secret_santa
├─ src/templates/                 # templates Tera (base.html.tera, ...)
└─ README.md
```

Remarques rapides :

- src/state.rs : AppState expose :
  - db_pool: pool SQLite (r2d2)
  - is_authenticated: Mutex<bool>
  - current_user: Mutex<Option<i64>>
  - current_access_code: Mutex<Option<AccessCode>> — stocke l'access code authentifié (id, name, code, active)
- Lors du POST /login, si le code existe et est actif, l'application :
  - marque `is_authenticated = true`
  - stocke l'AccessCode complet dans `AppState.current_access_code` (Mutex<Option<AccessCode>>)
  - tente de résoudre et stocker un `current_user` (id) si pertinent
- src/auth.rs contient le request guard `AuthenticatedUser` utilisé pour protéger les routes admin/API.
- Templates Tera sont présentes dans src/templates/ et utilisées pour les pages admin / UI.

## Dépendances principales

- rocket = "0.5.1" (feature "json")
- rusqlite, r2d2, r2d2_sqlite
- serde (derive)
- tera (templates)
- regex (validation côté serveur)

(Voir Cargo.toml pour la liste complète.)

## Routes importantes

- Pages public
  - GET  /                      → page d'accueil (login si non authentifié)
  - POST /login                 → login (champ `password` contenant un access code actif)

- Pages authentifiées
  - POST /logout                → logout
  - GET  /secret_santa          → page publique Secret Santa (exemple)
  - GET  /admin                 → interface admin (protégée)
  - GET  /admin/api/codes       → lister les access codes (JSON) — protégé
  - POST /admin/api/codes       → créer un code (JSON { code: String, active: bool, name: Option<String> }) — protégé
  - PATCH /admin/api/codes/<id> → mettre à jour `active` / `name` — protégé
  - DELETE /admin/api/codes/<id>→ supprimer un code — protégé

## Utilisation courante

1. Build & run (macOS / Linux) :

   ```bash
   cargo build
   cargo run
   ```

2. Par défaut l'application écoute sur `http://localhost:8000` (config Rocket par défaut).

3. Exemple : se connecter avec un code actif (champ `password` du formulaire). Après connexion, l'objet AccessCode authentifié est disponible via `state.current_access_code`.

Accéder au code authentifié depuis n'importe quel handler :

```rust
if let Ok(current) = state.current_access_code.lock() {
    if let Some(access) = &*current {
        // utiliser access.id, access.name, access.code, access.active
    }
}
```

## Sécurité & améliorations recommandées

- Ne pas stocker les codes en clair en production — hacher (argon2/bcrypt) et vérifier le hash.
- Remplacer le flag global `is_authenticated` par une gestion de session (cookies signés, JWT ou stockage server-side) pour supporter plusieurs sessions simultanées et CSRF.
- Activer HTTPS, validation stricte côté serveur et politique de contenu (CSP).
- Ajouter tests unitaires et d'intégration pour les routes et la logique DB.
- Limiter tentatives de connexion / logging pour audit.

## Notes

- Le stockage de l'AccessCode dans AppState est volontairement simple pour cet exemple. Pour une application multi-utilisateur ou distribuée, préférez des sessions signées ou un store de sessions.
