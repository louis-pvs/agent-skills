# SOLID Design Principles Reference

Comprehensive heuristics, detection patterns, and refactoring guidelines for the SOLID software design principles.

---

## 1. Single Responsibility Principle (SRP)

> **"A class/module should have one, and only one, reason to change."**

### SRP Detection Heuristics & Anti-Patterns

- **God Objects / Bloated Classes**: Classes exceeding 300+ lines or containing 10+ public methods spanning unrelated domain domains (e.g., handling database SQL + HTTP requests + JSON serialization + business validation).
- **Mixed Abstraction Levels**: High-level workflow orchestration mixed directly with low-level file I/O or byte manipulation.
- **Multiple Stakeholder Dependencies**: A class that changes when UI designers, database admins, or billing managers change requirements.

### SRP Code Smell Example

```python
# BAD: Violates SRP
class UserManager:
    def create_user(self, user_data: dict):
        # 1. Validation
        if not user_data.get("email"):
            raise ValueError("Email required")
        # 2. Database persistence
        db.execute("INSERT INTO users VALUES (...)")
        # 3. Notification dispatch
        send_email(user_data["email"], "Welcome!")
        # 4. Audit logging
        with open("/var/log/audit.log", "a") as f:
            f.write(f"Created user {user_data['email']}\n")
```

### SRP Refactoring Pattern

Decompose by responsibility into focused, single-purpose components (e.g., `UserValidator`, `UserRepository`, `UserNotificationService`).

---

## 2. Open/Closed Principle (OCP)

> **"Software entities should be open for extension, but closed for modification."**

### OCP Detection Heuristics & Anti-Patterns

- **Cascading `if/elif` or `switch` statements**: Long conditional blocks checking type codes, enums, or strings that require modifying existing code whenever a new type is added.
- **Hardcoded Type Checks**: `isinstance(obj, TypeA)` or `type(x) == TypeB` scatter-spread throughout business logic.
- **Direct Instantiation**: Classes creating concrete dependency instances using `new` / `Type()` directly inside core business logic instead of receiving factories or injected strategies.

### OCP Code Smell Example

```python
# BAD: Violates OCP - Adding a new payment type requires editing PaymentProcessor
class PaymentProcessor:
    def process(self, payment_type: str, amount: float):
        if payment_type == "credit_card":
            self._pay_credit_card(amount)
        elif payment_type == "paypal":
            self._pay_paypal(amount)
        elif payment_type == "crypto":
            self._pay_crypto(amount)
        else:
            raise ValueError("Unsupported payment method")
```

### OCP Refactoring Pattern

Extract strategy interfaces or polymorphic handlers. Register new handlers without altering existing execution engines.

---

## 3. Liskov Substitution Principle (LSP)

> **"Subtypes must be substitutable for their base types without altering program correctness."**

### LSP Detection Heuristics & Anti-Patterns

- **Type Checking Before Calling Subclass Methods**: Code that checks `if isinstance(sub, ConcreteClass)` before invoking subclass specific methods.
- **Throwing `NotImplementedError`**: Derived classes overriding base class methods only to throw unsupported operation exceptions.
- **Strengthened Preconditions or Weakened Postconditions**: Derived classes requiring more strict arguments or producing looser outputs than the interface contract guarantees.

### LSP Code Smell Example

```python
# BAD: Violates LSP - Square breaks Rectangle invariant
class Rectangle:
    def set_width(self, w: float):
        self.w = w

    def set_height(self, h: float):
        self.h = h


class Square(Rectangle):
    def set_width(self, w: float):
        self.w = w
        self.h = w  # Unexpected side effect mutating height!
```

### LSP Refactoring Pattern

Prefer composition over inheritance or extract narrower, distinct interfaces.

---

## 4. Interface Segregation Principle (ISP)

> **"Clients should not be forced to depend upon interfaces that they do not use."**

### ISP Detection Heuristics & Anti-Patterns

- **Fat Interfaces**: Interfaces defining 15+ methods where implementing classes frequently leave half of them empty or stubbed out (`pass` / `return None`).
- **Polluted Dependencies**: A light consumer module importing a giant header/interface containing dozens of irrelevant method declarations.

### ISP Code Smell Example

```python
# BAD: Violates ISP - Worker interface forces Robot to implement eat()
class Worker(Protocol):
    def work(self) -> None: ...
    def eat(self) -> None: ...


class Robot:
    def work(self) -> None:
        print("Working")

    def eat(self) -> None:
        raise NotImplementedError("Robots don't eat")
```

### ISP Refactoring Pattern

Split monolithic interfaces into small, cohesive, role-specific protocols (`Workable`, `Feedable`).

---

## 5. Dependency Inversion Principle (DIP)

> **"High-level modules should not depend on low-level modules. Both should depend on abstractions."**

### DIP Detection Heuristics & Anti-Patterns

- **Direct Imports of Infrastructure Concrete Classes**: High-level domain services importing `psycopg2`, `boto3`, or specific database drivers directly.
- **Tightly-Coupled Instantiation**: Domain services calling `self.db = PostgresDatabase()` inside `__init__` rather than accepting an abstracted `DatabaseInterface` parameter.
- **Untestable Core Logic**: Inability to unit test core logic without mocking raw network sockets or real disk I/O.

### DIP Code Smell Example

```python
# BAD: Violates DIP - OrderService tightly coupled to PostgresDatabase
from infrastructure.postgres import PostgresDatabase


class OrderService:
    def __init__(self):
        self.db = PostgresDatabase()  # Concrete dependency!
```

### DIP Refactoring Pattern

Inject abstract dependencies (`RepositoryProtocol`, `NotifierInterface`) via constructor injection.
