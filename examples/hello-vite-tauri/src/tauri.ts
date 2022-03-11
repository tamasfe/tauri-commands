import { invoke } from "@tauri-apps/api";
/**
 * A reply for hello.
 */
export interface HelloReply {
/**
 * The message to be written to the console.
 */
message: string;
}
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
 *  Send a friendly message and receive a reply.
 * 
 */
export function hello(request: HelloRequest,): Promise<HelloReply> {return invoke('hello', {_1: request,});}
export function addNumbers(_1: number,_2: number,): Promise<number> {return invoke('add numbers', {_1: _1,_2: _2,});}
/**
 *  Commands defined as functions have to be generic over the runtime.
 * 
 */
export function showWindow(): Promise<null> {return invoke('show_window', {});}
