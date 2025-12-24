# MongoDB Connections

Securely store and connect to MongoDB databases with encrypted credentials.

## Installation

```bash
pip install "pypangolin[mongodb]"
```

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import MongoDBAsset

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

MongoDBAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_mongo",
    connection_string="mongodb://mongo.example.com:27017/admin",
    credentials={
        "username": "dbuser",
        "password": "securepassword123"
    },
    store_key=True,  # Or False for user-managed
    description="Production MongoDB database"
)
```

## Connecting to Database

```python
mongo_client = MongoDBAsset.connect(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_mongo"
)

# Use the MongoDB client
db = mongo_client["mydb"]
collection = db["mycollection"]
documents = collection.find().limit(10)

for doc in documents:
    print(doc)

mongo_client.close()
```

## Connection String Formats

```
mongodb://host:port/database
mongodb://host:port  # No specific database
mongodb+srv://cluster.mongodb.net/database  # MongoDB Atlas
```

## Important Notes

- Use `/admin` database for authentication with root credentials
- MongoDB credentials are inserted into the connection string automatically
- For MongoDB Atlas, include the full connection string with `+srv`
