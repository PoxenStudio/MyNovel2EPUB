import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

interface Props {
  onClose: () => void;
}

export function AboutDialog({ onClose }: Props) {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion(null));
  }, []);

  return (
    <div className="modal modal-open">
      <div className="modal-box">
        <div className="flex items-center gap-4">
          <img src="/poxenstudio.png" alt="PoxenStudio" className="h-16 w-16 rounded-box object-contain" />
          <div>
            <h3 className="text-lg font-bold">小说Text文件转EPUB电子书</h3>
            {version && <p className="text-xs text-base-content/60">版本 {version}</p>}
          </div>
        </div>
        <p className="mt-4 text-sm">
          欢迎访问我们的
          <a href="https://mybooks.top" target="_blank" rel="noopener noreferrer" className="link link-primary">
            产品首页
          </a>
          了解个人电子书库 MyBooks。
        </p>
        <p className="mt-2 text-sm">
          如果觉得我们这个工具有用，可以到
          <a
            href="https://github.com/PoxenStudio/MyNovel2EPUB"
            target="_blank"
            rel="noopener noreferrer"
            className="link link-primary"
          >
            Github项目
          </a>
          上点亮一个 Star 或者提交你的问题, 或欢迎提交你的样式定义。
        </p>
        <p className="mt-2 text-sm">
          特别感谢@Yc提供的产品建议！
        </p>

        <div className="modal-action">
          <button type="button" className="btn" onClick={onClose}>
            关闭
          </button>
        </div>
      </div>
    </div>
  );
}
