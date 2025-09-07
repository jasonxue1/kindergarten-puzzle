import React from "react";
import { strings, type Lang } from "./i18n";

export const TutorialModal: React.FC<{
  lang: Lang;
  onClose: () => void;
}> = ({ lang, onClose }) => {
  const isZh = lang === "zh";

  return (
    <div
      className="scrim"
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 30,
      }}
      onClick={onClose}
    >
      <div
        className="card"
        style={{
          width: 720,
          maxWidth: "92vw",
          padding: "20px 24px",
          maxHeight: "80vh",
          overflowY: "auto",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          <h2 style={{ margin: "0 0 12px 0", flex: 1 }}>
            {isZh ? "教程 / 快速上手" : "Tutorial / Quick Start"}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="icon-btn"
            title={isZh ? "关闭" : "Close"}
          >
            <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
              <path d="M18.3 5.71L12 12l6.3 6.29-1.41 1.42L10.59 13.4 4.29 19.7 2.88 18.29 9.17 12 2.88 5.71 4.29 4.29l6.3 6.3 6.29-6.3z" />
            </svg>
            <span>{isZh ? "关闭" : "Close"}</span>
          </button>
        </div>

        {/* Goal */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>{isZh ? "目标" : "Goal"}</h3>
          <p style={{ margin: 0 }}>
            {isZh
              ? "拖动、旋转和翻转拼块，使其在不相互重叠的情况下拼入棋盘形状内。"
              : "Drag, rotate, and flip pieces to fit them into the board shape without overlapping."}
          </p>
        </section>

        {/* Basic manipulation */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "基本操作" : "Basic Manipulation"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "拖拽：用鼠标左键拖动拼块（按住并移动）。"
                : "Drag: Hold left mouse button and move a piece."}
            </li>

            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "翻转：按 F 镜像当前选中的最上层拼块。"
                : "Flip: Press F to mirror the topmost selected piece."}
            </li>
          </ul>
        </section>

        {/* Rotation + speed */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "旋转与速度" : "Rotation & Speed"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "长按 Q/E 连续旋转（Q 逆时针，E 顺时针）。"
                : "Hold Q/E to rotate continuously (Q counter‑clockwise, E clockwise)."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "按 S 切换快/慢模式；顶部第二行可精确设置快/慢速度（度/秒）。"
                : "Press S to toggle fast/slow; use the second toolbar row to set exact fast/slow speeds (deg/s)."}
            </li>
          </ul>
        </section>

        {/* Constraints */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "移动限制" : "Movement Constraints"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "按 L 开/关限制：开启后移动时不允许与棋盘边界或其它拼块交叉。"
                : "Press L to toggle restriction: when on, movement disallows crossing the board border or other pieces."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "按住 Shift 可临时启用限制（松开即关闭）。"
                : "Hold Shift to temporarily enable restriction while moving."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "状态栏会显示当前 锁定/速度 状态。"
                : "The status bar shows current Lock/Speed state."}
            </li>
          </ul>
        </section>

        {/* Toolbar */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>{isZh ? "工具栏" : "Toolbar"}</h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "返回主页：回到站点根目录。"
                : "Home: Return to site root."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "重开：恢复初始布局与设置。"
                : "Reset: Restore initial layout and settings."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "下载蓝图：导出蓝图风格的 PNG。"
                : "Download Blueprint: Export a blueprint‑style PNG."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "加载本地 JSON：加载本地拼图 JSON 文件。"
                : "Load JSON: Load a local puzzle JSON file."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "语言/主题：切换界面语言与明暗主题。"
                : "Language/Theme: Switch UI language and theme."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "速度设置：在第二行分别调整“快/慢”的旋转速度。"
                : "Speed: Adjust ‘Fast’ and ‘Slow’ rotation speeds on the second row."}
            </li>
          </ul>
        </section>

        {/* Loading puzzles */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "加载拼图" : "Loading Puzzles"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "在主页选择内置拼图，或点击“浏览 puzzle 目录”。"
                : "Choose a built‑in puzzle on the home screen, or click ‘Browse puzzle directory’."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "也可通过“加载 JSON”导入本地拼图文件（支持完整 puzzle 或 counts+shapes 格式）。"
                : "You can also ‘Load JSON’ to import a local puzzle file (supports full puzzle or counts+shapes format)."}
            </li>
          </ul>
        </section>

        {/* Notes and status */}
        <section style={{ marginBottom: 14 }}>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "备注与状态" : "Notes & Status"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "底部“备注”区域会显示当前拼图的说明（若提供）。"
                : "The ‘Notes’ area at the bottom shows puzzle‑specific info when available."}
            </li>
            <li style={{ padding: "6px 0" }}>
              {isZh
                ? "顶部状态栏实时显示锁定与速度模式。"
                : "The top status bar updates with Lock and Speed mode."}
            </li>
          </ul>
        </section>

        {/* Shortcuts summary */}
        <section>
          <h3 style={{ margin: "8px 0" }}>
            {isZh ? "快捷键汇总" : "Shortcuts"}
          </h3>
          <ul className="chooser">
            <li style={{ padding: "6px 0" }}>
              Q / E —{" "}
              {isZh ? "连续旋转（逆/顺）" : "continuous rotate (CCW/CW)"}
            </li>
            <li style={{ padding: "6px 0" }}>
              S — {isZh ? "切换快/慢模式" : "toggle fast/slow mode"}
            </li>
            <li style={{ padding: "6px 0" }}>
              L — {isZh ? "切换移动限制" : "toggle movement restriction"}
            </li>
            <li style={{ padding: "6px 0" }}>
              Shift — {isZh ? "临时启用限制" : "temporary restriction"}
            </li>
            <li style={{ padding: "6px 0" }}>
              F — {isZh ? "镜像翻转" : "mirror flip"}
            </li>
          </ul>
        </section>
      </div>
    </div>
  );
};
