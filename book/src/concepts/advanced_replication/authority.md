# Authority

Networked entities can be simulated on a client or on a server.
We define by 'Authority' the decision of which **peer is simulating an entity**.
The authoritative peer (client or server) is the only one that is allowed to send replication updates for an entity, and it won't accept updates from a non-authoritative peer.

Only **one peer** can be the authority over an entity at a given time.


### Benefits of distributed client-authority

Client authority means that the client is directly responsible for simulating an entity and sending 
replication updates for that entity.

Cons:
  - high exposure to cheating.
  - lower latency
Pros:
  - less CPU load on the server since the client is simulating some entities


### How it works

We have 2 components:
- `HasAuthority`: this is a marker component that you can use as a filter in queries
  to check if the current peer has authority over the entity.
  - on clients:
    - a client will not accept any replication updates from the server if it has `HasAuthority` for an entity
    - a client will send replication updates for an entity only if it has `HasAuthority` for that entity
  - on server:
    - this component is just used as an indicator for convenience, but the server can still send replication
      updates even if it doesn't have `HasAuthority` for an entity. (because it's broadcasting the updates coming
      from a client)
- `AuthorityPeer`: this component is only present on the server, and it indicates to the server which
  peer currently holds authority over an entity. (`None`, `Server` or a `Client`).
  The server will only accept replication updates for an entity if the sender matches the `AuthorityPeer`.

### Authority Transfer

On the server, you can use the `EntityCommand` `transfer_authority` to transfer the authority for an entity to a different peer.
The command is simply `commands.entity(entity).transfer_authority(new_owner)` to transfer the authority of `entity` to the `AuthorityPeer` `new_owner`.

Under the hood, authority transfers do two things:
- on the server, the transfer is applied immediately (i.e. the `HasAuthority` and `AuthorityPeer` components are updated instantly)
- than the server sends messages to clients to notify them of an authority change. Upon receiving the message, the client will add or remove the `HasAuthority` component as needed.

### Caveats

- There could be a time where both the client and server have authority at the same time
  - server is transferring authority from itself to a client: there is a period of time where
    no peer has authority, which is ok.
  - server is transferring authority from a client to itself: there is a period of time where
    both the client and server have authority. The client's updates won't be accepted by the server because it has authority, and the server's updates won't be accepted by the client because it 
    has authority, so no updates will be applied.
    
  - server is transferring authority from client C1 to client C2:
    - if C1 receives the message first, then for a short period of time no client has authority, which is ok
    - if C2 receives the message first, then for a short period of time both clients have authority. However the `AuthorityPeer` is immediately updated on the server, so the server will only 
      accept updates from C2, and will discard the updates from C1.

TODO:
- maybe let the client always accept updates from the server, even if the client has `HasAuthority`? What is the goal of disallowing the client to accept updates from the server if it has
`HasAuthority`?
- maybe include a timestamp/tick to the `ChangeAuthority` messages so that any in-flight replication updates can be handled correctly? 
- maybe have an API `request_authority` where the client requests the authority? and receives a response from the server telling it if the request is accepted or not?