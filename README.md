<!-- PROJECT BANNER -->
<p align="center">
    <img src="./front/public/logo.png" alt="data-go-fast Logo" width="100" />
</p>

<p align="center">
    <strong>data-go-fast</strong> — An Open-Source data processing app by Polytech Montpellier students
    <br/>
    <em>Cloud‑native • Built in Rust</em>
</p>

## 🏗️ Architecture (Rust Workspace)

**data-go-fast** is built using an event-driven, microservices architecture. To maximize code reuse and maintainability, the project is structured as a **Rust Cargo Workspace** containing multiple interconnected crates:

*   🌐 **`server` (The Web API):** A fast, asynchronous HTTP backend. It is responsible for handling incoming REST requests, validating JWT authentication, streaming file uploads directly to our S3, and queuing jobs. It is completely *stateless* and can be scaled horizontally.
*   🥦 **`worker` (The Orchestrator):** A dedicated background processing engine (affectionately named *Broccoli*). It continuously listens to the Redis message queues, streams raw data back from S3, infers data types, and executes heavy processing pipelines (filtering, aggregation, grouping) without blocking the main API.
*   📦 **`common` (Shared Library):** The core library shared by both the `server` and the `worker`. It acts as the single source of truth for our domain models (e.g., `Job`, `Datasource`, `User`) and contains all infrastructure configurations (PostgreSQL connections via SQLx, Redis clients, S3 configuration). This prevents code duplication across the microservices.
*   🎨 **`front`:** Contains the static assets, documentation images, and visual identity (such as our mascot logo) used across the project and documentation.

### 🔄 How it works
When a user uploads a large dataset, the **`server`** streams it to S3, logs the metadata in PostgreSQL, and pushes a task to Redis. It then immediately returns a `202 Accepted` response. Meanwhile, the **`worker`** picks up the task from Redis, processes the heavy data asynchronously in the background, and updates the database once the job is complete.
