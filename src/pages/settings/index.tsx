import { useState } from "react";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/react";
import { Settings, Puzzle, Settings2, Info, Server } from "lucide-react";
import { useTranslation } from "react-i18next";

import SettingsPanel from "@/components/Settings/SettingsPanel";
import GeneralSettings from "@/components/Settings/GeneralSettings";
import AboutView from "@/components/Settings/AboutView";
import Cloud from "@/components/Cloud/Cloud.tsx";
import Footer from "@/components/Footer";
import { useAppStore } from "@/stores/appStore";

function SettingsPage() {
  const { t } = useTranslation();

  const tabs = [
    { name: t("settings.tabs.general"), icon: Settings },
    { name: t("settings.tabs.extensions"), icon: Puzzle },
    { name: t("settings.tabs.connect"), icon: Server },
    { name: t("settings.tabs.advanced"), icon: Settings2 },
    { name: t("settings.tabs.about"), icon: Info },
  ];

  const tabIndex = useAppStore((state) => state.tabIndex);
  const [defaultIndex, setDefaultIndex] = useState<number>(tabIndex);

  return (
    <div>
      <div className="min-h-screen pb-8 bg-white dark:bg-gray-900 text-gray-900 dark:text-white">
        <div className="max-w-6xl mx-auto p-4">
          {/* <div className="flex items-center justify-center mb-2">
              <h1 className="text-xl font-bold">Coco Settings</h1>
            </div> */}

          <TabGroup
            selectedIndex={defaultIndex}
            onChange={(index) => {
              setDefaultIndex(index);
            }}
          >
            <TabList className="flex space-x-1 rounded-xl bg-gray-100 dark:bg-gray-800 p-1">
              {tabs.map((tab) => (
                <Tab
                  key={tab.name}
                  className={({ selected }) =>
                    `w-full rounded-lg py-2.5 text-sm font-medium leading-5
                      ${
                        selected
                          ? "bg-white dark:bg-gray-700 shadow text-gray-900 dark:text-white"
                          : "text-gray-700 dark:text-gray-400 hover:bg-white/[0.12] hover:text-gray-900 dark:hover:text-white"
                      }
                      flex items-center justify-center space-x-2 focus:outline-none`
                  }
                >
                  <tab.icon className="w-4 h-4" />
                  <span>{tab.name}</span>
                </Tab>
              ))}
            </TabList>

            <TabPanels className="mt-2">
              <TabPanel>
                <SettingsPanel title="">
                  <GeneralSettings />
                </SettingsPanel>
              </TabPanel>
              <TabPanel>
                <SettingsPanel title="">
                  <div className="text-gray-600 dark:text-gray-400">
                    {t("settings.tabs.extensionsContent")}
                  </div>
                </SettingsPanel>
              </TabPanel>
              <TabPanel>
                <Cloud />
              </TabPanel>
              <TabPanel>
                <SettingsPanel title="">
                  <div className="text-gray-600 dark:text-gray-400">
                    {t("settings.tabs.advancedContent")}
                  </div>
                </SettingsPanel>
              </TabPanel>
              <TabPanel>
                <SettingsPanel title="">
                  <AboutView />
                </SettingsPanel>
              </TabPanel>
            </TabPanels>
          </TabGroup>
        </div>
      </div>
      <Footer />
    </div>
  );
}

export default SettingsPage;
