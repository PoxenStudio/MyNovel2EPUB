import { useBookContext } from "../../context/BookContext";
import { STEP_ORDER } from "../../utils/constants";
import type { WizardStep } from "../../types";

const STEP_LABELS: Record<WizardStep, string> = {
  import: "导入分析",
  config: "视觉配置",
  export: "编译导出",
};

export function StepperNav() {
  const { state } = useBookContext();
  const currentIndex = STEP_ORDER.indexOf(state.step);

  return (
    <ul className="steps w-full py-4">
      {STEP_ORDER.map((step, index) => (
        <li
          key={step}
          data-content={index + 1}
          className={`step ${index <= currentIndex ? "step-primary" : ""}`}
        >
          {STEP_LABELS[step]}
        </li>
      ))}
    </ul>
  );
}
