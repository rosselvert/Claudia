import { Navigate, Route, Routes } from "react-router-dom";
import AccountPage from "./AccountPage";
import AdminPage from "./AdminPage";
import StorePage from "./StorePage";
import { GlobalUi } from "./components";
import { StoreProvider } from "./StoreContext";

export default function App() {
  return <StoreProvider>
    <Routes>
      <Route path="/" element={<StorePage />} />
      <Route path="/account" element={<AccountPage />} />
      <Route path="/admin" element={<AdminPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
    <GlobalUi />
  </StoreProvider>;
}
