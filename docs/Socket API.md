# Socket API
This file describes the endpoints needed to implement the functionality of this program.

## Unix Domain Socket
This project uses Unix Domain sockets for IPC.  The daemon should be running and listening for the connection from the frontend app when the app is launched.  Both programs will connect to the domain socket at "/tmp/ehr.sock".

## Endpoints

### Get chains
Returns an array of all the chains on this system, by name and id.
- action: **get_chains**
- parameters: None
- response: 
    ```
    {
        [
            {
                id: int,
                name: string
            }
        ]
    }
    ```

### Create chain
Create a new chain respresenting a person. Returns the new id.
- action: **create_chain**
- parameters:
    ```
    {
        firstName: string, (required)
        lastName: string, (required)
        dateOfBirth: string, (required)
        sex: string, (required)
        weight: int,
        height: int
    }
    ```

- response:
    ```
    {
        id : int
    }
    ```

### Get Chain
Get all information for a single chain. Return base info, list of records, and list of providers.
- action: **get_chain**
- parameters:
    ```
    {
        id: int
    }
    ```

- response:
    ```
    {
        id: int,
        info: {
            firstName: string,
            lastName: string,
            dateOfBirth: string,
            sex: string,
            weight: int,
            height: int
        },
        providers: [{
            id: int,
            name: string,
            ipAddress: string
        }],
        records: [{
            id: int,
            subject: string,
            provider: name
        }]
    }
    ```

### Update Info
Update base info for a chain representing a person. Return ok: true if success.
- action: **update_info**
- parameters: 
    ```
    {
        id: int,
        weight: int,
        height: int
    }
    ```
- response: 
    ```
    {
        ok: boolean
    }
    ```

### Add Record
Add record to a chain given inputs.  Return ok: true if success.
- action: **add_record**
- parameters: 
    ```
    {
        id: int,
        subject: string,
        body: string
    }
    ```
- response:
    ```
    {
        ok: boolean
    }
    ```

### Get Record
Get an individual record.
- action: **get_record**
- parameters:
    ```
    {
        chain_id: int,
        record_id: int
    }
    ```
- response:
    ```
    {
        date: string,
        subject: string,
        providerName: string,
        body: string
    }
    ```

### Get Provider
Get information for individual provider.
- action **get_provider**
- parameters: 
    ```
    {
        chain_id: int,
        provider_id: int
    }
    ```
- response:
    ```
    {
        name: string,
        ipAddress: string,
        recordCount: int
    }
    ```

### Revoke Provider
Revoke access from a provider to a given chain. Return ok: true if success.
- action: **revoke_provider**
- parameters: 
    ```
    {
        chain_id: int,
        provider_id: int
    }
    ```
- response:
    ```
    {
        ok: boolean
    }
    ```

### Add Provider
Search by ip address and add provider to user. Return ok: true if success.
- action: **add_provider**
- parameters:
    ```
    {
        id: int,
        name: string,
        ipAddress: string
    }
    ```
- response: 
    ```
    {
        ok: boolean
    }
    ```