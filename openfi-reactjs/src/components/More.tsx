import { ArrowRight, ChevronDown, FingerprintIcon, PhilippinePeso, RefreshCcw, Router, Shuffle, SunMoonIcon } from "lucide-react";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "../components/ui/popover"
import { type ReactElement } from "react";
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

function More({ children }: { children: ReactElement }) {
    function toggleTheme() {
        const theme = window.localStorage.getItem("theme") || "light";
        const n = (theme === "light" ? "dark" : "light")
        document.documentElement.classList = n;
        window.localStorage.setItem("theme", n);
    }

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
                            <span>10.0.0.1</span>
                        </div>
                        <div className="p-3 flex gap-4 border-b border-foreground/5">
                            <FingerprintIcon size="17" />
                            <span>aa:bb:cc:dd:ee</span>
                        </div>

                        <div className="p-2 font-semibold text-xs">Actions</div>
                        <div onClick={toggleTheme} className="p-3 flex gap-4 border-b  border-t border-foreground/5 cursor-pointer hover:bg-primary/5 duration-75">
                            <SunMoonIcon size="17" />
                            <span className="grow">Toggle Theme</span>
                            <Shuffle size="17" className="mt-px" />
                        </div>

                        <Drawer>
                            <DrawerTrigger nativeButton={false} render={
                                <div className="p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-primary/5 duration-75">
                                    <PhilippinePeso size="17" className="mt-px" />
                                    <span className="grow">View Rates</span>
                                    <ChevronDown size="17" className="mt-px" />
                                </div>
                            }>Open</DrawerTrigger>
                            <DrawerContent className="flex items-center w-full p-6">
                                <div className="max-w-md w-full">
                                    <Table>
                                        <TableCaption>A list of your recent invoices.</TableCaption>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead className="w-[100px]">Invoice</TableHead>
                                                <TableHead>Status</TableHead>
                                                <TableHead>Method</TableHead>
                                                <TableHead className="text-right">Amount</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            <TableRow>
                                                <TableCell className="font-medium">INV001</TableCell>
                                                <TableCell>Paid</TableCell>
                                                <TableCell>Credit Card</TableCell>
                                                <TableCell className="text-right">$250.00</TableCell>
                                            </TableRow>
                                        </TableBody>
                                    </Table>
                                </div>
                            </DrawerContent>
                        </Drawer>

                        <div className="p-3 flex gap-4 border-b border-foreground/5 cursor-pointer hover:bg-primary/5 duration-75">
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