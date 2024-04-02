# Eigen FileStorage

**eigen-filestorage** is a Rust library designed to streamline file and directory operations for both local file systems and Amazon S3 object storage. It provides a unified interface that simplifies the shift between on-premise and cloud storage, offering flexibility for cross-platform and cloud-native developments.

### Features

- Unified API for seamless file and directory management across local and S3 storage.
- Easy storage backend switching, enhancing application adaptability.
- Cross-platform support, ideal for diverse Rust application scenarios.

### Use Cases

- Developing applications that integrate with multiple storage backends.
- Managing files in cloud-native and microservice architectures.
- Simplifying data migration and rapid prototyping processes.

**eigen-filestorage** enables Rust developers to focus on building their applications, without the overhead of managing storage differences.

> ⚠️ To reduce the impact of migrating storage solutions on existing projects, we have implemented synchronous calls to the S3 API, accepting reduced performance for increased reliability and simplicity in integration.
