import { Link, useLocation } from "wouter";
import {
    HomeIcon,
    MonitorSmartphone,
    PhilippinePeso,
    Cog,
    User,
    SunMoonIcon,
    Shuffle,
    LogOut,
    ChevronRight
} from "lucide-react";
import { cn } from "#lib/utils";
import { Button } from "#components/ui/button";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "#components/ui/popover"
import { toggleTheme } from "#components/More";

function AdminLayout({ children }: { children: React.ReactNode }) {
    const [location, setLocation] = useLocation();

    const navItems = [
        { path: "/dashboard", icon: HomeIcon, label: "Home" },
        { path: "/users", icon: MonitorSmartphone, label: "Users" },
        { path: "/vouch_rates", icon: PhilippinePeso, label: "Vouch & Rates" },
        { path: "/settings", icon: Cog, label: "Settings" }
    ];

    const headText: Record<string, string[]> = {
        "/dashboard": ["Dashboard", "View analytics and monitor system status."],
        "/users": ["Users", "Manage and monitor user accounts and information."],
        "/vouch_rates": ["Vouchers & Rates", "Create and manage access vouchers, rates, and pricing options."],
        "/settings": ["Settings", "Manage system configurations, WiFi access points, and quality of service."],
    }

    const [pageTitle, pageDesc] = headText[location] ?? ["", ""];

    return (
        <div className="h-screen [@supports(height:100dvh)]:h-dvh w-screen relative bg-background text-foreground overflow-hidden">
            <div className="h-full w-full overflow-y-auto pb-24 pt-24">
                <div className="max-w-5xl mx-auto h-full">
                    {children}
                </div>
            </div>

            <div className="absolute top-1 w-full">
                <div className="max-w-5xl mx-auto p-4 py-3 bg-background/5 backdrop-blur-md flex">
                    <div className="grow">
                        <h1 className="text-foreground font-semibold">{pageTitle}</h1>
                        <p className="text-xs text-foreground/70 w-[75%]">{pageDesc}</p>
                    </div>

                    <Popover>
                        <PopoverTrigger render={
                            <Button size="icon" variant="outline" className="border-primary/30 mt-1">
                                <User />
                            </Button>
                        }>
                            Open Popover
                        </PopoverTrigger>
                        <PopoverContent align="end" side="bottom" className="p-0 w-48 shadow-none overflow-hidden">
                            <div>
                                <button onClick={toggleTheme} className="w-full text-left p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-primary/5 transition-colors duration-75">
                                    <SunMoonIcon size="17" />
                                    <span className="grow">Toggle Theme</span>
                                    <Shuffle size="17" className="mt-px" />
                                </button>
                                <button onClick={() => {
                                    window.localStorage.removeItem("adminkey");
                                    setLocation("/")
                                }} className="w-full text-left p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-destructive/10 transition-colors duration-75 bg-destructive/5 text-destructive">
                                    <LogOut size="17" />
                                    <span className="grow">Logout</span>
                                    <ChevronRight size="17" className="mt-px" />
                                </button>
                            </div>
                        </PopoverContent>
                    </Popover>
                </div>
                <span className="block absolute w-full border-b" />
            </div>

            <div className="absolute bottom-6 w-full pointer-events-none">
                <div className="w-full flex justify-center">
                    <nav className="pointer-events-auto flex items-center gap-1 p-1 rounded-full bg-card/80 backdrop-blur-2xl border-2 border-foreground/5">
                        {navItems.map((item) => {
                            const isActive = location === item.path;
                            const Icon = item.icon;

                            return (
                                <Link key={item.path} href={item.path}>
                                    <div
                                        className={cn(
                                            "flex items-center gap-2 cursor-pointer transition-all duration-300",
                                            isActive
                                                ? "text-primary bg-foreground/2 px-5 py-2.5 rounded-full"
                                                : "text-muted-foreground hover:text-primary p-2.5 rounded-full"
                                        )}
                                    >
                                        <Icon size={20} strokeWidth={isActive ? 2.5 : 2} />
                                        {isActive && (
                                            <span className="text-sm font-medium pr-1">
                                                {item.label}
                                            </span>
                                        )}
                                    </div>
                                </Link>
                            );
                        })}
                    </nav>
                </div>
            </div>
        </div>
    );
}

export default AdminLayout;