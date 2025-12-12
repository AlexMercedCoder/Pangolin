# Tag Management

Pangolin supports tagging commits to mark specific points in history, such as releases or stable versions.

## Data Model

A `Tag` consists of:
- `name`: Unique identifier for the tag within a catalog.
- `commit_id`: The ID of the commit the tag points to.

## API Endpoints

### Create Tag
`POST /api/v1/tags`

**Request Body:**
```json
{
  "name": "v1.0",
  "commit_id": "uuid-of-commit"
}
```

**Response:**
```json
{
  "name": "v1.0",
  "commit_id": "uuid-of-commit"
}
```

### List Tags
`GET /api/v1/tags`

**Response:**
```json
[
  {
    "name": "v1.0",
    "commit_id": "uuid-of-commit"
  }
]
```

### Delete Tag
`DELETE /api/v1/tags/:name`

**Response:** `204 No Content`
