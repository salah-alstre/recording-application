import { Component, useEffect, type ErrorInfo, type ReactNode } from "react";
import { AlertTriangle, RotateCcw } from "lucide-react";
import { t } from "@/i18n";
import { navlog } from "@/lib/navlog";

interface Props { name: string; children: ReactNode }
interface State { error: Error | null; attempt: number }

/** Keeps one failing page from blanking the app: shows the real error with a retry. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, attempt: 0 };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`[NAV] page "${this.props.name}" crashed while rendering:`, error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="page">
          <div className="banner error" role="alert">
            <AlertTriangle size={18} style={{ color: "var(--danger)", flex: "none", marginTop: 2 }} />
            <div style={{ flex: 1 }}>
              <div className="b-title">{t("error.pageTitle")}</div>
              <div className="b-body">{t("error.pageBody")}</div>
              <div className="b-body mono" dir="ltr" style={{ marginTop: 6, userSelect: "text" }}>{this.state.error.message}</div>
            </div>
            <button className="btn sm" onClick={() => this.setState((s) => ({ error: null, attempt: s.attempt + 1 }))}><RotateCcw size={14} />{t("common.retry")}</button>
          </div>
        </div>
      );
    }
    return <MountLog name={this.props.name} key={this.state.attempt}>{this.props.children}</MountLog>;
  }
}

function MountLog({ name, children }: { name: string; children: ReactNode }) {
  useEffect(() => {
    navlog("Mount:", name);
    return () => navlog("Unmount:", name);
  }, [name]);
  return <>{children}</>;
}
