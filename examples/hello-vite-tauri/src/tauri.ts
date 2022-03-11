import { invoke } from "@tauri-apps/api";
/**
 * The request data.
 */
export interface HelloRequest {
/**
 * This message is printed to stdout.
 */
message: string;
}
/**
 * A reply for hello.
 */
export interface HelloReply {
/**
 * The message to be written to the console.
 */
message: string;
}
export function hello(request: HelloRequest,): Promise<HelloReply> {return invoke('hello', {_1: request,});}
