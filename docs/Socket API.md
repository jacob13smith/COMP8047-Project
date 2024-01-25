# Socket API
This file describes the endpoints needed to implement the functionality of this program.

# Unix Domain Socket
This project uses Unix Domain sockets for IPC.  The daemon should be running and listening for the connection from the frontend app when the app is launched.  Both programs will connect to the domain socket at "/tmp/ehr.sock".

# Endpoints

- Get chains
    - action: **get_chains**
    - parameters: None
    - response: 
        ```
        {
            [
                {
                    chain_id: int,
                    name: string
                }
            ]
        }
        ```