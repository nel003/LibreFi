import { ArrowRight, ChevronDown, FingerprintIcon, PhilippinePeso, RefreshCcw, Router, Shuffle, SunMoonIcon } from "lucide-react";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "../components/ui/popover"
import { useCallback, useEffect, useState, type ReactElement } from "react";
import {
    Drawer,
    DrawerContent,
    DrawerTrigger,
} from "../components/ui/drawer"
import {
    Table,
    TableBody,
    TableCaption,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "../components/ui/table"
import type { UserType } from "../types/user";
import type { RateType } from "../types/rates";
import { formatSeconds } from "#lib/utils";

function toggleTheme() {
    const theme = window.localStorage.getItem("theme") || "light";
    const n = (theme === "light" ? "dark" : "light")
    document.documentElement.classList = n as string;
    window.localStorage.setItem("theme", n);
}

function More({ user, children }: { user: UserType | undefined, children: ReactElement }) {
    const [rates, setRates] = useState<RateType[] | null>();

    const getRates = useCallback(async () => {
        const res = await fetch("/rates");
        if (res.status === 200) {
            const json = await res.json();
            setRates(json.rows);
        }
    }, []);

    useEffect(() => {
        (() => {
            getRates()
        })()
    }, [getRates])

    return (
        <>
            <Popover>
                <PopoverTrigger render={children}>
                    Open Popover
                </PopoverTrigger>
                <PopoverContent className="p-0 w-48">
                    <div>
                        <div className="p-2 font-semibold text-xs">Details</div>
                        <div className="p-3 flex gap-4 border-b border-t border-foreground/5">
                            <Router size="17" />
                            <span>{user?.ip}</span>
                        </div>
                        <div className="p-3 flex gap-4 border-b border-foreground/5">
                            <FingerprintIcon size="17" />
                            <span>{user?.mac}</span>
                        </div>

                        <div className="p-2 font-semibold text-xs">Actions</div>
                        <button onClick={toggleTheme} className="w-full text-left p-3 flex gap-4 border-b  border-t border-foreground/5 cursor-pointer hover:bg-primary/5 transition-colors duration-75">
                            <SunMoonIcon size="17" />
                            <span className="grow">Toggle Theme</span>
                            <Shuffle size="17" className="mt-px" />
                        </button>

                        <Drawer>
                            <DrawerTrigger nativeButton={false} render={
                                <div className="p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-primary/5 transition-colors duration-75">
                                    <PhilippinePeso size="17" className="mt-px" />
                                    <span className="grow">View Rates</span>
                                    <ChevronDown size="17" className="mt-px" />
                                </div>
                            }>Open</DrawerTrigger>
                            <DrawerContent className="flex items-center w-full p-6">
                                <div className="max-w-md min-w-xs w-full">
                                    <Table>
                                        <TableCaption>A list of rates.</TableCaption>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead className="text-foreground/20">ID</TableHead>
                                                <TableHead>Price</TableHead>
                                                <TableHead className="text-right">Time</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {rates && rates.map(r => (
                                                <TableRow key={r.id}>
                                                    <TableCell className="font-medium text-foreground/20">{r.id}</TableCell>
                                                    <TableCell>{r.rate.price}</TableCell>
                                                    <TableCell className="text-right">{formatSeconds(r.rate.time)}</TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                </div>
                            </DrawerContent>
                        </Drawer>

                        <div className="p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-primary/5 transition-colors duration-75">
                            <RefreshCcw size="17" />
                            <span className="grow">Fix Connection</span>
                            <ArrowRight size="17" className="mt-px" />
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
        </>
    )
}

export default More;