# DDD Patterns & Modeling Heuristics

Detailed reference guide for categorizing domain building blocks: Entities, Value Objects, Aggregates, Domain Events, and Ubiquitous Language.

---

## 1. Entities vs. Value Objects

| Trait          | Entity                                        | Value Object                                        |
| :------------- | :-------------------------------------------- | :-------------------------------------------------- |
| **Identity**   | Has unique ID (`OrderId`, `UserId`)           | Defined purely by its attribute values              |
| **Equality**   | Equal if IDs match, even if attributes differ | Equal if all attributes match                       |
| **Mutability** | State changes over time                       | Immutable (always produce a new instance on change) |
| **Lifecycle**  | Has explicit lifecycle and state transitions  | Instantiated, passed, replaced                      |
| **Examples**   | `Order`, `CustomerAccount`, `Subscription`    | `Money`, `EmailAddress`, `DateRange`, `Address`     |

### Value Object Design Rules

1. **Self-Validating Construction**: Validate invariants inside the constructor/initializer. Throw domain exceptions immediately if invalid.
2. **Immutability**: Use frozen dataclasses or read-only properties.
3. **Behavior Over Data**: Include domain operations on value objects (e.g. `money.add(other_money)` returning a new `Money`).

---

## 2. Ubiquitous Language Guidelines

- **No Tech Speak in Domain**: Avoid names like `UserDataProcessor`, `OrderDBRow`, `OrderDTO` inside the domain layer. Use `Order`, `Customer`, `LineItem`.
- **Verb-Noun Alignment**: Name commands and events after business domain verbs (`SubmitOrder`, `OrderPlaced`, `RefundIssued`).
- **Glossary Table Structure**:

| Domain Term    | Definition                                           | Anti-Pattern / Terms to Avoid | Context  |
| :------------- | :--------------------------------------------------- | :---------------------------- | :------- |
| **Subscriber** | Active paying member with an open subscription.      | User, Client, Account         | Billing  |
| **Line Item**  | An individual product quantity unit inside an order. | ItemRow, ProductRef           | Checkout |

---

## 3. Domain Events

Domain Events signal significant domain state changes that have already occurred (past tense):

- `OrderPlaced`
- `PaymentReceived`
- `SubscriptionCancelled`

### Event Guidelines

- Domain events are **immutable records**.
- Contain minimal necessary payload (aggregate ID, timestamp, event ID, mutated state payload).
- Published after aggregate invariants pass cleanly.
