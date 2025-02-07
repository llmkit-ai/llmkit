## Sqlite

1. Have sqlx-cli installed
2. Create a new DB if needed
```
sqlx database create
```
3. To create a new migration file run
```
sqlx migrate add <name>
```
4. To run migration file (create fresh db) run
```
sqlx migrate run
```
5. Run prepare
```
cargo sqlx prepare --check
```


### Sample prompt: 
```
<!-- role:system -->
you are a helpful assistant

<!-- role:user -->
sup dude
```
