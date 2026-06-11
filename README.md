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
*   🦜 **`worker` (The Orchestrator):** A dedicated background processing engine powered by **Apalis** + Redis. It continuously listens to the Redis job queue, downloads raw data from S3, and executes heavy processing pipelines (filtering, aggregation, grouping) without blocking the main API.
*   📦 **`common` (Shared Library):** The core library shared by both the `server` and the `worker`. It acts as the single source of truth for our domain models (e.g., `Job`, `Datasource`, `User`) and contains all infrastructure configurations (PostgreSQL connections via SQLx, Redis clients, S3 configuration). This prevents code duplication across the microservices.
*   🎨 **`front`:** A React/TypeScript that provides the user interface for datasource management, job monitoring, and pipeline execution.

### 🔄 How it works
When a user uploads a dataset, the **`server`** streams it to **Garage S3**, logs the metadata in **PostgreSQL**, and pushes a job to **Redis**. The **`worker`** picks up the job from Redis, downloads the raw data from S3, executes the pipeline operation, uploads the result back to S3, and creates a new datasource entry in the database.



## 👷👷‍♀️ Contributors
- Nadia LAHYA ([@nadouulh](https://github.com/nadouulh))
- Charlotte LEWIS ([@c-r-lewis](https://github.com/c-r-lewis))
- Jessy LATMI ([@JyVers](https://github.com/JyVers))
- Eliott BASSIER ([@beliott](https://github.com/beliott))
