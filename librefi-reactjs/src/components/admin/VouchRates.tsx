
import { encrypt, formatSeconds, getAdminKey } from "#lib/utils";
import { useEffect, useState } from "react";
import {
    Table,
    TableBody,
    TableCaption,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "../ui/table"
import { Button } from "../ui/button";

import { useLocation } from "wouter";
import { Pencil, Plus } from "lucide-react";
import type { RateType } from "../../types/rates";
import CreateEditVoucher from "./CreateUpdateVoucher";
import DeleteVoucher from "./DeleteVoucher";
import CreateUpdateRate from "./CreateUpdateRate";
import DeleteRate from "./DeleteRate";

interface VoucherT {
    "code": string
    "price": number
    "time": number
    "used": boolean
    "censored": boolean
}

function censor(code: string) {
    return code.substring(0, 2) + "****"
}

function AdminVouchRates() {
    const [vouchers, setVouchers] = useState<VoucherT[] | null>();
    const [rates, setRates] = useState<RateType[] | null>();
    const [voucherPage, setVoucherPage] = useState(1);
    const [voucherTotalPages, setVoucherTotalPages] = useState(1);

    const [_, setLocation] = useLocation();

    async function getVouchers() {
        const res = await fetch("/api/admin/vouchers?payload=" + await encrypt(getAdminKey(), JSON.stringify({ page: voucherPage })));
        if (res.ok) {
            const json = await res.json();
            setVouchers(json.vouchers.map((v: VoucherT) => ({ ...v, censored: true })));
            setVoucherTotalPages(json.total_pages);
        } else {
            setLocation("/");
        }
    }

    async function getRates() {
        const res = await fetch("/api/rates");
        if (res.ok) {
            const json = await res.json();
            setRates(json.rows);
        }
    }

    useEffect(() => {
        getVouchers();
    }, [voucherPage])

    useEffect(() => {
        getRates();
    }, [])

    return (
        <div className="h-auto w-full pb-20">
            <div className="w-full flex flex-col md:flex-row">

                <div className="w-full md:w-1/2 border-b-4 md:border-r-4 md:border-b-0 border-foreground/3 p-4 ">
                    <div>
                        <Table>
                            {vouchers?.length !== 0 ? <></> : <TableCaption>No vouchers found.</TableCaption>}
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="text-xs font-bold">Code</TableHead>
                                    <TableHead>Price</TableHead>
                                    <TableHead>Time</TableHead>
                                    <TableHead className="flex justify-end items-center">
                                        <CreateEditVoucher getVouchers={getVouchers} btn={<Button size="xs" variant="secondary" className="text-emerald-500"><Plus /> New</Button>} />
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {vouchers && vouchers.map(v => (
                                    <TableRow key={v.code}>
                                        <TableCell onClick={() => setVouchers(vouchers.map((vo) => ({ ...vo, censored: v.code === vo.code ? !vo.censored : vo.censored })))} className="text-xs font-bold">
                                            {v.censored ? censor(v.code) : v.code}
                                        </TableCell>
                                        <TableCell>{v.price}</TableCell>
                                        <TableCell>{formatSeconds(v.time)}</TableCell>
                                        <TableCell className="flex justify-end gap-1">
                                            <CreateEditVoucher getVouchers={getVouchers} code={v.code} price={v.price} time={v.time} btn={<Button size="icon-xs" variant="outline" className="border-primary text-foreground/60"><Pencil /></Button>} />
                                            <DeleteVoucher getVouchers={getVouchers} code={v.code} />
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>

                        <div className="w-full flex justify-center">
                            <div className="flex items-center justify-between mt-4 px-2">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    disabled={voucherPage <= 1}
                                    onClick={() => setVoucherPage(p => Math.max(1, p - 1))}
                                >
                                    Prev
                                </Button>
                                <span className="text-sm text-muted-foreground font-medium">
                                    Page {voucherPage} of {voucherTotalPages}
                                </span>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    disabled={voucherPage >= voucherTotalPages}
                                    onClick={() => setVoucherPage(p => Math.min(voucherTotalPages, p + 1))}
                                >
                                    Next
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="w-full md:w-1/2 p-4">
                    <div>
                        <Table>
                            {rates?.length !== 0 ? <></> : <TableCaption>No rates found.</TableCaption>}
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="text-xs font-bold">Price</TableHead>
                                    <TableHead>Time</TableHead>
                                    <TableHead className="flex justify-end items-center">
                                        <CreateUpdateRate getRates={getRates} btn={<Button size="xs" variant="secondary" className="text-emerald-500"><Plus /> New</Button>} />
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {rates && rates.map(r => (
                                    <TableRow key={r.id}>
                                        <TableCell className="text-xs font-bold">
                                            {r.rate.price}
                                        </TableCell>
                                        <TableCell>{formatSeconds(r.rate.time)}</TableCell>
                                        <TableCell className="flex justify-end gap-1">
                                            <CreateUpdateRate getRates={getRates} id={r.id} price={r.rate.price} time={r.rate.time} btn={<Button size="icon-xs" variant="outline" className="border-primary text-foreground/60"><Pencil /></Button>} />
                                            <DeleteRate getRates={getRates} id={r.id} price={r.rate.price} />
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default AdminVouchRates;