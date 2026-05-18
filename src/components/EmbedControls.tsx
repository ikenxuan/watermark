import { useState, useRef } from "react";
import { Button, Spinner, TextArea } from "@heroui/react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Download } from "lucide-react";
import type { ResultData } from "../types";

interface EmbedControlsProps {
  file: File | null;
  result: ResultData | null;
  onResult: (result: ResultData) => void;
}

export default function EmbedControls({ file, result, onResult }: EmbedControlsProps) {
  const [watermarkText, setWatermarkText] = useState("");
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLDivElement>(null);

  const handleEmbed = async () => {
    if (!file || !watermarkText.trim()) {
      alert("请选择图片并输入水印文本");
      return;
    }
    setLoading(true);
    try {
      const arrayBuffer = await file.arrayBuffer();
      const imageBytes = Array.from(new Uint8Array(arrayBuffer));
      const response = await invoke<{
        image_bytes: number[];
        duration_ms: number;
      }>("embed_watermark", {
        imageBytes,
        watermarkText: watermarkText.trim(),
      });
      const blob = new Blob([new Uint8Array(response.image_bytes)], {
        type: "image/png",
      });
      const dataUrl = URL.createObjectURL(blob);
      onResult({
        type: "image",
        dataUrl,
        durationMs: response.duration_ms,
        filename: `watermarked_${file.name.replace(/\.[^/.]+$/, "")}.png`,
      });
    } catch (err) {
      alert(`嵌入失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleSaveAs = async () => {
    if (!result || result.type !== "image") return;
    const imageResult = result as Extract<typeof result, { type: "image" }>;
    const filePath = await save({
      defaultPath: imageResult.filename,
      filters: [{ name: "PNG Image", extensions: ["png"] }],
    });
    if (filePath) {
      const response = await fetch(imageResult.dataUrl);
      const blob = await response.blob();
      const buffer = await blob.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));
      await invoke("save_file", { path: filePath, data: bytes });
    }
  };

  return (
    <div className="flex flex-col gap-3">
      {/* Watermark Input */}
      {file && (
        <div ref={inputRef}>
          <TextArea
            fullWidth
            aria-label="水印文本"
            placeholder='{"a":1715424000000,"b":"v1.0.0","c":"user123"}'
            value={watermarkText}
            onChange={(e) => setWatermarkText(e.target.value)}
            rows={2}
            variant="secondary"
          />
        </div>
      )}

      {/* Action Button */}
      {file && (
        <Button
          variant="primary"
          size="sm"
          fullWidth
          onPress={handleEmbed}
          isDisabled={!file || !watermarkText.trim() || loading}
        >
          {loading ? <Spinner color="current" size="sm" /> : "嵌入水印"}
        </Button>
      )}

      {/* Save As Button */}
      {result?.type === "image" && (
        <Button variant="primary" size="sm" fullWidth onPress={handleSaveAs}>
          <Download size={14} className="mr-1" />
          另存为
        </Button>
      )}
    </div>
  );
}
