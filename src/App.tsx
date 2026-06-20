import { BookProvider, useBookContext } from "./context/BookContext";
import { AppHeader } from "./components/layout/AppHeader";
import { StepperNav } from "./components/layout/StepperNav";
import { ImportStep } from "./components/steps/ImportStep";
import { ConfigStep } from "./components/steps/ConfigStep";
import { ExportStep } from "./components/steps/ExportStep";

function WizardContent() {
  const { state } = useBookContext();

  switch (state.step) {
    case "import":
      return <ImportStep />;
    case "config":
      return <ConfigStep />;
    case "export":
      return <ExportStep />;
  }
}

function App() {
  return (
    <BookProvider>
      <div className="min-h-screen bg-base-100">
        <AppHeader />
        <div className="mx-auto max-w-5xl px-6">
          <StepperNav />
          <main>
            <WizardContent />
          </main>
        </div>
      </div>
    </BookProvider>
  );
}

export default App;
