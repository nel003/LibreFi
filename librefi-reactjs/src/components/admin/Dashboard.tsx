import { useEffect, useState } from "react";
import {
    Coins,
    Cpu,
    MemoryStick,
    HardDrive,
    Activity
} from "lucide-react";
import { encrypt, getAdminKey } from "#lib/utils";
import { useLocation } from "wouter";

interface DashBoardT {
    cpu_used_pct: number
    ram_used_pct: number
    ram_free_pct: number
    storage_used_pct: number
    storage_free_pct: number
    total_coins: number
    total_users: number
    total_active_users: number
}

function AdminDashboard() {
    const [_, setLocation] = useLocation();
    const [data, setData] = useState<DashBoardT | null>();

    async function getDashboard() {
        const res = await fetch("/api/admin/dashboard?payload=" + await encrypt(getAdminKey(), JSON.stringify({ "id": 1 })));
        if (res.ok) {
            const json = await res.json();
            setData(json)
        } else {
            setLocation("/");
        }
    }

    useEffect(() => {
        getDashboard();
        const timer = setInterval(async () => {
            getDashboard();
        }, 3000)

        return () => clearInterval(timer)
    }, []);


    return (
        <div className="p-4 pt-0 flex flex-col gap-4 pb-20">
            <div className="grid grid-cols-1 md:grid-cols-2 md:grid-rows-6 flex-1 border border-border rounded-4xl">
                <div className="col-span-1 md:row-span-3 border-b md:border-r border-border p-6 flex flex-col justify-center items-center text-center relative">
                    <div className="absolute top-7 left-5 text-foreground/2 dark:text-foreground/6">
                        <Activity size={150} />
                    </div>
                    <div className="absolute top-7 right-5 text-foreground/2 dark:text-foreground/6 ">
                        <Activity size={150} />
                    </div>
                    <h2 className="text-muted-foreground font-medium text-lg mt-4">Active Users</h2>
                    <p className="text-7xl font-bold mt-2 text-primary">{data?.total_active_users}</p>
                    <p className="text-sm text-muted-foreground mt-4">Out of <span className="font-bold">{data?.total_users}</span> total users</p>
                </div>

                <div className="col-span-1 md:row-span-3 border-b md:border-b-0 md:border-r border-border p-6 flex flex-col justify-center items-center text-center relative pb-10">
                    <div className="absolute top-7 left-5 text-foreground/2 dark:text-foreground/6">
                        <Coins size={150} />
                    </div>
                    <div className="absolute top-7 right-5 text-foreground/2 dark:text-foreground/6">
                        <Coins size={150} />
                    </div>
                    <h2 className="text-muted-foreground font-medium text-lg mt-4">Total Revenue</h2>
                    <div className="flex items-end gap-1 mt-2">
                        <span className="text-3xl font-semibold text-muted-foreground mb-1">₱</span>
                        <p className="text-6xl font-bold text-foreground">
                            {data?.total_coins.toLocaleString()}
                        </p>
                    </div>
                </div>

                <div className="col-span-1 md:row-span-2 md:col-start-2 md:row-start-1 border-b border-border p-5 flex items-center gap-5">
                    <div className="shrink-0 text-foreground/40">
                        <Cpu size={28} />
                    </div>
                    <div className="flex-1 pr-2">
                        <h3 className="text-muted-foreground text-sm font-medium">CPU Usage</h3>
                        <p className="text-2xl font-bold text-foreground mt-1">{data?.cpu_used_pct}%</p>
                        <div className="w-full bg-foreground/10 h-2 rounded-full mt-3 overflow-hidden">
                            <div
                                className="bg-foreground h-full rounded-full transition-all duration-1000"
                                style={{ width: `${data?.cpu_used_pct}%` }}
                            />
                        </div>
                    </div>
                </div>

                <div className="col-span-1 md:row-span-2 md:col-start-2 md:row-start-3 border-b border-border p-5 flex items-center gap-5">
                    <div className="shrink-0 text-foreground/40">
                        <MemoryStick size={28} />
                    </div>
                    <div className="flex-1 pr-2">
                        <h3 className="text-muted-foreground text-sm font-medium">RAM Usage</h3>
                        <p className="text-2xl font-bold text-foreground mt-1">{data?.ram_used_pct}%</p>
                        <div className="w-full bg-foreground/10 h-2 rounded-full mt-3 overflow-hidden">
                            <div
                                className="bg-foreground h-full rounded-full transition-all duration-1000"
                                style={{ width: `${data?.ram_used_pct}%` }}
                            />
                        </div>
                    </div>
                </div>

                <div className="col-span-1 md:row-span-2 md:col-start-2 md:row-start-5 p-5 flex items-center gap-5">
                    <div className="shrink-0 text-foreground/40">
                        <HardDrive size={28} />
                    </div>
                    <div className="flex-1 pr-2">
                        <h3 className="text-muted-foreground text-sm font-medium">Storage</h3>
                        <p className="text-2xl font-bold text-foreground mt-1">{data?.storage_used_pct}%</p>
                        <div className="w-full bg-foreground/10 h-2 rounded-full mt-3 overflow-hidden">
                            <div
                                className="bg-foreground h-full rounded-full transition-all duration-1000"
                                style={{ width: `${data?.storage_used_pct}%` }}
                            />
                        </div>
                    </div>
                </div>

            </div>
        </div>
    );
}

export default AdminDashboard;