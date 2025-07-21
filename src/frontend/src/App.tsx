import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/HomePage";
import { CollectionPage } from "./pages/CollectionPage";
import {
  TokenPage,
  DataPage,
  ActivityPage,
  ProfilePage,
  SupportPage,
  SettingPage,
  ResourcePage,
} from "./pages";

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/token" element={<TokenPage />} />
          <Route path="/data" element={<DataPage />} />
          <Route path="/activity" element={<ActivityPage />} />
          <Route path="/profile" element={<ProfilePage />} />
          <Route path="/support" element={<SupportPage />} />
          <Route path="/setting" element={<SettingPage />} />
          <Route path="/resource" element={<ResourcePage />} />
          <Route path="/collection/:id" element={<CollectionPage />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
