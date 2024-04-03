# Future Enhancements
This document is a collection of ideas for enhancements to the system, if it were to be fully built out as a usable EHR system.

- Database versioning and upgrades.
  - Currently, the database tables are built once when the app is run. If the system were deployed live, it would be ideal to have a way to upgrade/downgrade the database with data migration using SQL scripts.

- Unique User ID system across nodes
  - Currently, when a new patient file is created, we use UUIDv4 generation and hope that no other user in the system has that id. 
  - It would be better to have a system that combined the provider's public key/peer id with a unique local id to guarantee uniqueness across all nodes.

- Long-running daemon connection
  - Currently, once the daemon and client make the Socket connection, there is not capability for reconnecting, if for example the client is closed and reopened.
  - Ideally, the daemon would always be running and the client would open, connect via socket, and close connection when user is done, with the daemon accepting the connection every time.
  - You would need this long running functionality to support the network of nodes in a real environment.

- Unit testing for all internally created functions
  - For a production-level system, I would want to have unit tests to test all components and integrations with eachother. This would be too much work for the time constraint of this project.