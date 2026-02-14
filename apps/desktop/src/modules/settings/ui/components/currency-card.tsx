import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";

interface CurrencyCardProps {
	settings: Settings;
	onUpdate: (input: UpdateSettingsInput) => void;
}

const CURRENCY_OPTIONS = [
	{ value: "AUD", label: "AUD (A$) - Australian Dollar" },
	{ value: "USD", label: "USD ($) - US Dollar" },
	{ value: "EUR", label: "EUR (\u20AC) - Euro" },
	{ value: "GBP", label: "GBP (\u00A3) - British Pound" },
	{ value: "JPY", label: "JPY (\u00A5) - Japanese Yen" },
	{ value: "CAD", label: "CAD (C$) - Canadian Dollar" },
	{ value: "NZD", label: "NZD (NZ$) - New Zealand Dollar" },
	{ value: "CHF", label: "CHF (Fr) - Swiss Franc" },
	{ value: "CNY", label: "CNY (\u00A5) - Chinese Yuan" },
	{ value: "HKD", label: "HKD (HK$) - Hong Kong Dollar" },
	{ value: "SGD", label: "SGD (S$) - Singapore Dollar" },
	{ value: "SEK", label: "SEK (kr) - Swedish Krona" },
	{ value: "NOK", label: "NOK (kr) - Norwegian Krone" },
	{ value: "DKK", label: "DKK (kr) - Danish Krone" },
	{ value: "KRW", label: "KRW (\u20A9) - South Korean Won" },
	{ value: "INR", label: "INR (\u20B9) - Indian Rupee" },
	{ value: "BRL", label: "BRL (R$) - Brazilian Real" },
	{ value: "ZAR", label: "ZAR (R) - South African Rand" },
	{ value: "MXN", label: "MXN (Mex$) - Mexican Peso" },
	{ value: "TWD", label: "TWD (NT$) - Taiwan Dollar" },
	{ value: "THB", label: "THB (\u0E3F) - Thai Baht" },
	{ value: "MYR", label: "MYR (RM) - Malaysian Ringgit" },
	{ value: "PHP", label: "PHP (\u20B1) - Philippine Peso" },
	{ value: "IDR", label: "IDR (Rp) - Indonesian Rupiah" },
	{ value: "PLN", label: "PLN (z\u0142) - Polish Zloty" },
	{ value: "CZK", label: "CZK (K\u010D) - Czech Koruna" },
	{ value: "HUF", label: "HUF (Ft) - Hungarian Forint" },
	{ value: "ILS", label: "ILS (\u20AA) - Israeli Shekel" },
	{ value: "TRY", label: "TRY (\u20BA) - Turkish Lira" },
	{ value: "AED", label: "AED (AED) - UAE Dirham" },
] as const;

export function CurrencyCard({ settings, onUpdate }: CurrencyCardProps) {
	return (
		<SettingsCard
			title="Currency"
			description="Set your preferred currency for price comparisons"
		>
			<div className="flex items-center justify-between">
				<Label htmlFor="preferred-currency">Preferred Currency</Label>
				<Select
					value={settings.preferred_currency}
					onValueChange={(value) => onUpdate({ preferred_currency: value })}
				>
					<SelectTrigger id="preferred-currency" className="w-64">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{CURRENCY_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		</SettingsCard>
	);
}
