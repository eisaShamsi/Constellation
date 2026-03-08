/**
 * Built-in skills that ship with Constellation.
 */

import type { SkillDefinition } from './types';

export const builtinSkills: SkillDefinition[] = [
	{
		id: 'summarize',
		name: 'Summarize Note',
		name_ar: 'تلخيص الملاحظة',
		description: 'Generate a concise summary of any note',
		description_ar: 'إنشاء ملخص موجز لأي ملاحظة',
		icon: '✦',
		category: 'analysis',
		inputs: [
			{
				type: 'textarea',
				key: 'note_content',
				label: 'Note content',
				label_ar: 'محتوى الملاحظة',
				placeholder: 'Paste your note content here...',
				placeholder_ar: 'الصق محتوى ملاحظتك هنا...',
				required: true
			}
		],
		systemPrompt: 'You are a note summarization assistant. Be concise and preserve key information.',
		promptTemplate: 'Summarize the following note concisely. Highlight key points, decisions, and action items if any:\n\n{{note_content}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'qa',
		name: 'Smart Q&A',
		name_ar: 'أسئلة وأجوبة ذكية',
		description: 'Ask questions about your notes and get AI-powered answers',
		description_ar: 'اطرح أسئلة حول ملاحظاتك واحصل على إجابات ذكية',
		icon: '?',
		category: 'research',
		inputs: [
			{
				type: 'textarea',
				key: 'context',
				label: 'Note content (context)',
				label_ar: 'محتوى الملاحظة (سياق)',
				placeholder: 'Paste the note(s) you want to ask about...',
				placeholder_ar: 'الصق الملاحظات التي تريد السؤال عنها...',
				required: true
			},
			{
				type: 'text',
				key: 'question',
				label: 'Your question',
				label_ar: 'سؤالك',
				placeholder: 'What would you like to know?',
				placeholder_ar: 'ماذا تريد أن تعرف؟',
				required: true
			}
		],
		systemPrompt: 'You are a knowledgeable assistant. Answer questions based on the provided note content. If the answer is not in the notes, say so.',
		promptTemplate: 'Based on the following notes:\n\n{{context}}\n\nAnswer this question: {{question}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'write',
		name: 'Writing Assistant',
		name_ar: 'مساعد الكتابة',
		description: 'Expand, rewrite, or improve your text',
		description_ar: 'توسيع أو إعادة كتابة أو تحسين نصك',
		icon: '✎',
		category: 'writing',
		inputs: [
			{
				type: 'textarea',
				key: 'text',
				label: 'Your text',
				label_ar: 'نصك',
				placeholder: 'Enter the text you want to improve...',
				placeholder_ar: 'أدخل النص الذي تريد تحسينه...',
				required: true
			},
			{
				type: 'select',
				key: 'action',
				label: 'Action',
				label_ar: 'الإجراء',
				options: [
					{ value: 'improve', label: 'Improve writing', label_ar: 'تحسين الكتابة' },
					{ value: 'expand', label: 'Expand', label_ar: 'توسيع' },
					{ value: 'shorten', label: 'Make concise', label_ar: 'اختصار' },
					{ value: 'formal', label: 'Make formal', label_ar: 'جعله رسمي' },
					{ value: 'casual', label: 'Make casual', label_ar: 'جعله عفوي' }
				],
				required: true
			}
		],
		systemPrompt: 'You are a professional writing assistant. Maintain the original meaning while applying the requested changes.',
		promptTemplate: 'Please {{action}} the following text:\n\n{{text}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'autolink',
		name: 'Auto-Linker',
		name_ar: 'الربط التلقائي',
		description: 'Discover connections between notes across vaults',
		description_ar: 'اكتشف الروابط بين الملاحظات عبر الخزائن',
		icon: '⟡',
		category: 'organization',
		inputs: [
			{
				type: 'textarea',
				key: 'note_content',
				label: 'Note content',
				label_ar: 'محتوى الملاحظة',
				placeholder: 'Paste your note content...',
				placeholder_ar: 'الصق محتوى ملاحظتك...',
				required: true
			},
			{
				type: 'textarea',
				key: 'other_titles',
				label: 'Other note titles (one per line)',
				label_ar: 'عناوين الملاحظات الأخرى (واحد في كل سطر)',
				placeholder: 'List your other note titles here...',
				placeholder_ar: 'أدرج عناوين ملاحظاتك الأخرى هنا...',
				required: true
			}
		],
		systemPrompt: 'You are a knowledge management assistant. Analyze content and find meaningful connections to other notes.',
		promptTemplate: 'Analyze the following note and suggest which of the listed notes it should be linked to, with a brief reason for each connection.\n\nNote content:\n{{note_content}}\n\nAvailable notes to link to:\n{{other_titles}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'translate',
		name: 'Translate Note',
		name_ar: 'ترجمة الملاحظة',
		description: 'Translate notes between languages',
		description_ar: 'ترجمة الملاحظات بين اللغات',
		icon: '🌐',
		category: 'writing',
		inputs: [
			{
				type: 'textarea',
				key: 'text',
				label: 'Text to translate',
				label_ar: 'النص المراد ترجمته',
				placeholder: 'Paste the text to translate...',
				placeholder_ar: 'الصق النص المراد ترجمته...',
				required: true
			},
			{
				type: 'select',
				key: 'target_lang',
				label: 'Target language',
				label_ar: 'اللغة المستهدفة',
				options: [
					{ value: 'English', label: 'English', label_ar: 'الإنجليزية' },
					{ value: 'Arabic', label: 'Arabic', label_ar: 'العربية' },
					{ value: 'French', label: 'French', label_ar: 'الفرنسية' },
					{ value: 'Spanish', label: 'Spanish', label_ar: 'الإسبانية' },
					{ value: 'German', label: 'German', label_ar: 'الألمانية' },
					{ value: 'Chinese', label: 'Chinese', label_ar: 'الصينية' },
					{ value: 'Japanese', label: 'Japanese', label_ar: 'اليابانية' }
				],
				required: true
			}
		],
		systemPrompt: 'You are a professional translator. Translate accurately while maintaining the tone and formatting of the original.',
		promptTemplate: 'Translate the following text to {{target_lang}}. Preserve any markdown formatting:\n\n{{text}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'meeting',
		name: 'Meeting Notes',
		name_ar: 'ملاحظات الاجتماع',
		description: 'Structure and organize raw meeting notes',
		description_ar: 'تنظيم وهيكلة ملاحظات الاجتماعات',
		icon: '📋',
		category: 'organization',
		inputs: [
			{
				type: 'textarea',
				key: 'raw_notes',
				label: 'Raw meeting notes',
				label_ar: 'ملاحظات الاجتماع الخام',
				placeholder: 'Paste your raw meeting notes...',
				placeholder_ar: 'الصق ملاحظات اجتماعك الخام...',
				required: true
			}
		],
		systemPrompt: 'You are a meeting notes organizer. Structure raw notes into a clean, actionable format.',
		promptTemplate: 'Organize the following raw meeting notes into a structured format with:\n- Summary\n- Key Discussion Points\n- Decisions Made\n- Action Items (with owners if mentioned)\n- Follow-ups\n\nRaw notes:\n{{raw_notes}}',
		output: 'markdown',
		builtin: true
	},
	{
		id: 'chart',
		name: 'Chart Generator',
		name_ar: 'مولد الرسوم البيانية',
		description: 'Create chart descriptions from your note data',
		description_ar: 'إنشاء وصف رسوم بيانية من بيانات ملاحظاتك',
		icon: '📊',
		category: 'generation',
		inputs: [
			{
				type: 'textarea',
				key: 'data',
				label: 'Data or note content',
				label_ar: 'البيانات أو محتوى الملاحظة',
				placeholder: 'Paste data or notes to visualize...',
				placeholder_ar: 'الصق البيانات أو الملاحظات للتصور...',
				required: true
			},
			{
				type: 'select',
				key: 'chart_type',
				label: 'Chart type',
				label_ar: 'نوع الرسم البياني',
				options: [
					{ value: 'bar', label: 'Bar Chart', label_ar: 'رسم بياني شريطي' },
					{ value: 'pie', label: 'Pie Chart', label_ar: 'رسم بياني دائري' },
					{ value: 'timeline', label: 'Timeline', label_ar: 'خط زمني' },
					{ value: 'mindmap', label: 'Mind Map', label_ar: 'خريطة ذهنية' }
				],
				required: true
			}
		],
		systemPrompt: 'You are a data visualization assistant. Extract structured data from notes and describe it in a format suitable for chart rendering.',
		promptTemplate: 'Analyze the following data and create a {{chart_type}} representation. Output the data as a structured JSON object with labels and values, followed by a text description:\n\n{{data}}',
		output: 'json',
		builtin: true
	},
	{
		id: 'research',
		name: 'Research Assistant',
		name_ar: 'مساعد البحث',
		description: 'Analyze and synthesize information across multiple notes',
		description_ar: 'تحليل وتجميع المعلومات من ملاحظات متعددة',
		icon: '🔬',
		category: 'research',
		inputs: [
			{
				type: 'textarea',
				key: 'notes',
				label: 'Notes to analyze (paste multiple)',
				label_ar: 'الملاحظات للتحليل (الصق عدة ملاحظات)',
				placeholder: 'Paste the notes you want to analyze...',
				placeholder_ar: 'الصق الملاحظات التي تريد تحليلها...',
				required: true
			},
			{
				type: 'text',
				key: 'focus',
				label: 'Research focus (optional)',
				label_ar: 'محور البحث (اختياري)',
				placeholder: 'What aspect should the analysis focus on?',
				placeholder_ar: 'على أي جانب يجب أن يركز التحليل؟',
				required: false
			}
		],
		systemPrompt: 'You are a research analyst. Synthesize information from multiple sources and provide structured insights.',
		promptTemplate: 'Analyze the following notes and provide:\n- Key Themes\n- Common Patterns\n- Contradictions or Gaps\n- Synthesis & Insights\n{{#if focus}}\nFocus your analysis on: {{focus}}\n{{/if}}\n\nNotes:\n{{notes}}',
		output: 'markdown',
		builtin: true
	}
];
