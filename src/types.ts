export interface ImageResult {
  type: "image";
  dataUrl: string;
  durationMs: number;
  filename: string;
}

export interface TextResult {
  type: "text";
  text: string;
  durationMs: number;
}

export type ResultData = ImageResult | TextResult;
