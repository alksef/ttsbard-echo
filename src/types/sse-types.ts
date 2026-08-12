/**
 * SSE Message Payload Types
 * These types define the structure of messages received from server through SSE
 */

export interface SSEMessage {
  type: string;
  payload: any;
  timestamp: number;
}

export interface TypingIndicatorPayload {
  userId?: string;
  isTyping: boolean;
  text?: string;
}

export interface TextPayload {
  id: string;
  content: string;
  source: string;
  timestamp: number;
  sender?: string;
}

export interface KeepalivePayload {
  lastReceived: number;
  serverTime: number;
}

export type SSEPayload = TypingIndicatorPayload | TextPayload | KeepalivePayload | string;

export interface SSEMessageEvent {
  id: string;
  data: SSEPayload;
  event: string;
}