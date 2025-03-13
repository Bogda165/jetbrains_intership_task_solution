# jetbrains_intership_task_solution
As I really enjoy programiring in rust, I will use it to implement this project.

## Implementation 
(In this section I will write all thoughts about this impls)
### Question 1: Why the data can be corrupted?
HTTP/1.1 use TCP so data can not be corrupted on the network layer. So the problem is probably on the server side. So there is nothing I can do minimize the amount of corruptions.

### Question2: Does server support pipelining?
- Status: Unknown
- If there server support pipelining, I would need to set up an alternative of dynamic sliding window, not to overcome the perfomace of the server(there I can use Range which is mentioned in the task)

### Optimizations
1. In real situation the size of transfered file can be big, so storing it in the memory would cost a lot. I see the solution in storing incoming data in the file on disk by some chunks.

### Plan
1. First of all I will solve the problem using crates.
   - I will probably choose reqwest for communicating witha  server and sha256, because I have already used them.
2. The next step would be to implement a client side of HTTPx protocol, probably I will just use HTTP1/1.
   1. As HTTP is an application layer protocol, and HTTP1/1 uses TCP to avoid using external crates I will use std::net, still I think tokio::net will suit more.
   2. First of all I will implement a simple one with a sender thread and a few recievers. Later I am going to improve it.
   