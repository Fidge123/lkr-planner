# Daylite Search & Filter API

This document describes how the Daylite `_search` endpoints work, based on the OpenAPI spec and official filter documentation. All searchable entity types (`/projects/_search`, `/contacts/_search`, etc.) share the same filter structure.

## Endpoint Shape

```
POST /{entity}/_search
```

Query parameters:

| Parameter      | Type    | Default | Description                          |
| -------------- | ------- | ------- | ------------------------------------ |
| `limit`        | integer | 50      | Maximum results to return            |
| `full-records` | boolean | —       | Include full record data in response |
| `start`        | integer | —       | Pagination start point (object ID)   |

Response:

```json
{
  "results": [
    { "self": "/v1/projects/3001", "name": "Rebranding Campaign" }
  ],
  "next": "/v1/projects/_search?start=3002"
}
```

## Filter Body Structure

### AND — plain object

Keys within a single JSON object are joined with logical **AND**:

```json
{ "name": { "contains": "Nord" }, "status": { "equal": "in_progress" } }
```

### OR — array of objects

Objects in a JSON array are joined with logical **OR**:

```json
[
  { "status": { "equal": "new_status" } },
  { "status": { "equal": "in_progress" } }
]
```

AND and OR can be combined — each array item is itself an AND-object:

```json
[
  { "name": { "contains": "Nord" }, "status": { "equal": "new_status" } },
  { "name": { "contains": "Nord" }, "status": { "equal": "in_progress" } }
]
```

An empty body `{}` returns all records (subject to `limit`).

## Operators by Field Type

| Field type          | Supported operators                                                                                                   |
| ------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `string`            | `equal`, `not_equal`, `any`, `not_any`, `blank`, `not_blank`, `starts_with`, `does_not_start_with`, `contains`, `does_not_contain`, `equal_ignore_case` |
| `integer`           | `equal`, `not_equal`, `less_than`, `less_than_equal`, `greater_than`, `greater_than_equal`, `in_range`, `not_in_range`, `any`, `not_any`, `blank`, `not_blank` |
| `date`              | `equal`, `not_equal`, `less_than`, `less_than_equal`, `greater_than`, `greater_than_equal`, `in_range`, `not_in_range`, `any`, `not_any`, `blank`, `not_blank` |
| `datetime`          | `less_than`, `less_than_equal`, `greater_than`, `greater_than_equal`, `in_range`, `not_in_range`, `blank`, `not_blank` |
| `boolean`           | `equal`, `not_equal`, `blank`, `not_blank`                                                                            |
| `reference`         | `equal`, `not_equal`, `any`, `not_any`, `blank`, `not_blank`                                                          |
| Minor types / Roles | `any`, `not_any`, `blank`, `not_blank`                                                                                |

> `blank` and `not_blank` ignore their operand value — `{"blank": true}` and `{"blank": false}` are equivalent.

> The `self` attribute is not supported in search filters.

## Filter Examples

### Single field, single value

```json
{ "category": { "equal": "Monteur" } }
```

### Multiple fields (AND)

```json
{ "name": { "contains": "Nord" }, "category": { "equal": "Überfällig" } }
```

### Multiple values for one field (OR via array)

```json
[
  { "status": { "equal": "new_status" } },
  { "status": { "equal": "in_progress" } }
]
```

### Multi-value keyword filter

```json
{ "keywords": { "any": ["Greek", "Norse"], "not_any": ["Comics"] } }
```

### Date range

```json
{ "due": { "less_than": "2026-04-30T00:00:00.000Z" } }
```

### Minor entity sub-filter (e.g. contacts on a project)

```json
{
  "contacts": {
    "any": { "contact": { "equal": "/v1/contacts/1001" } }
  }
}
```

## Usage in This Codebase

- **`search_projects_core`** (`src-tauri/src/integrations/daylite/projects.rs`) — sends a filter body built from `DayliteSearchInput`; when `statuses` is provided it builds an array-OR body, otherwise a plain object
- **`list_contacts_core`** (`src-tauri/src/integrations/daylite/contacts.rs`) — uses `{"category": {"equal": "Monteur"}}` to fetch only employee contacts
- **Overdue query** (BL-031) — will use `{"category": {"equal": "Überfällig"}}` for the default suggestions feature
