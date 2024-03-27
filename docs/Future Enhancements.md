# Future Enhancements
This document is a collection of ideas for enhancements to the system, if it were to be fully built out as a usable EHR system.

- Database versioning and upgrades.
  - Currently, the database tables are built once when the app is run. If the system were deployed live, it would be ideal to have a way to upgrade/downgrade the database with data migration using SQL scripts.

- Unique User ID system across nodes
  - Currently, when a new patient file is created, we use UUIDv4 generation and hope that no other user in the system has that id. 
  - It would be better to have a system that combined the provider's public key/peer id with a unique local id to guarantee uniqueness across all nodes.