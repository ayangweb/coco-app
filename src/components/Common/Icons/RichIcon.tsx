import { Folder } from "lucide-react";

import IconWrapper from "./IconWrapper";
import ThemedIcon from "./ThemedIcon";
import { useFindConnectorIcon } from "@/hooks/useFindConnectorIcon";
import { useAppStore } from "@/stores/appStore";

interface RichIconProps {
  item: any;
  className?: string;
  onClick: (e: React.MouseEvent) => void;
}

function RichIcon({ item, className, onClick }: RichIconProps) {
  const endpoint_http = useAppStore((state) => state.endpoint_http);

  const connectorSource = useFindConnectorIcon(item);
  const icons = connectorSource?.assets?.icons || {};

  // If the selectedIcon is a valid base64-encoded image
  const isBase64 = item?.rich_categories?.[0]?.icon?.startsWith("data:image/");
  if (isBase64) {
    return (
      <IconWrapper className={className} onClick={onClick}>
        <img
          className={className}
          src={item?.rich_categories?.[0]?.icon}
          alt="icon"
        />
      </IconWrapper>
    );
  }

  const selectedIcon = icons[item?.rich_categories?.[0]?.icon];

  if (!selectedIcon) {
    return (
      <IconWrapper className={className} onClick={onClick}>
        <ThemedIcon component={Folder} className={className} />
      </IconWrapper>
    );
  }

  if (
    selectedIcon.startsWith("http://") ||
    selectedIcon.startsWith("https://")
  ) {
    return (
      <IconWrapper className={className} onClick={onClick}>
        <img className={className} src={selectedIcon} alt="icon" />
      </IconWrapper>
    );
  } else {
    return (
      <IconWrapper className={className} onClick={onClick}>
        <img
          className={className}
          src={`${endpoint_http}${selectedIcon}`}
          alt="icon"
        />
      </IconWrapper>
    );
  }
}

export default RichIcon;
